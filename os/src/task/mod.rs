mod context;
mod id;
mod manager;
mod process;
mod processor;
mod signal;
mod switch;
#[allow(clippy::module_inception)]
mod task;

use self::id::TaskUserRes;
use crate::fs::proc::write_proc;
use crate::fs::{open_file, OpenFlags, ROOT_INODE};
use crate::sbi::shutdown;
use alloc::{sync::Arc, vec::Vec};
use lazy_static::*;
use log::{info, trace};
use manager::fetch_task;
use process::ProcessControlBlock;
use switch::__switch;

pub use context::TaskContext;
pub use id::{kstack_alloc, pid_alloc, KernelStack, PidHandle, IDLE_PID};
pub use manager::{add_task, pid2process, remove_from_pid2process, wakeup_task};
pub use processor::{
    current_kstack_top, current_process, current_task, current_trap_cx, current_trap_cx_user_va,
    current_user_token, run_tasks, schedule, take_current_task,
};
pub use signal::SignalFlags;
pub use task::{TaskControlBlock, TaskInfo, TaskStatus};

pub fn suspend_current_and_run_next() {
    // let cpid = current_process().clone().pid.0;
    // if cpid != 0 {
    //     info!(
    //         "kernel: pid[{}] suspend_current_and_run_next",
    //         current_process().clone().pid.0
    //     );
    // }
    // There must be an application running.
    let task = take_current_task().unwrap();

    // ---- access current TCB exclusively
    let mut task_inner = task.inner_exclusive_access(file!(), line!());
    let task_cx_ptr = &mut task_inner.task_cx as *mut TaskContext;
    // Change status to Ready
    task_inner.task_status = TaskStatus::Ready;
    let task_info = TaskInfo {
        pid: task.get_pid(),
        ppid: task.get_ppid(),
        status: task_inner.task_status,
        user_time: task_inner.user_time,
        kernel_time: task_inner.kernel_time,
        time_created: task_inner.time_created,
        first_time: task_inner.first_time,
    };
    drop(task_inner);
    // ---- release current TCB

    // push back to ready queue.
    add_task(task);

    write_proc(task_info);

    // jump to scheduling cycle
    schedule(task_cx_ptr);
}

/// This function must be followed by a schedule
pub fn block_current_task() -> *mut TaskContext {
    let task = take_current_task().unwrap();
    let mut task_inner = task.inner_exclusive_access(file!(), line!());
    task_inner.task_status = TaskStatus::Blocked;
    // let task_info = TaskInfo {
    //     pid: task.get_pid(),
    //     ppid: task.get_ppid(),
    //     status: task_inner.task_status,
    //     user_time: task_inner.user_time,
    //     kernel_time: task_inner.kernel_time,
    //     time_created: task_inner.time_created,
    // };
    // write_proc(task_info);

    &mut task_inner.task_cx as *mut TaskContext
}

pub fn block_current_and_run_next() {
    // info!(
    //     "kernel: pid[{}] block_current_and_run_next",
    //     current_process().clone().pid.0
    // );
    let task_cx_ptr = block_current_task();
    schedule(task_cx_ptr);
}

/// Exit the current 'Running' task and run the next task in task list.
pub fn exit_current_and_run_next(exit_code: i32) {
    // info!(
    //     "kernel: pid[{}] exit_current_and_run_next",
    //     current_process().clone().pid.0
    // );
    // println!(
    //     "kernel: pid[{}] exit with exit_code {}",
    //     current_process().clone().pid.0,
    //     exit_code
    // );
    let task = take_current_task().unwrap();
    let mut task_inner = task.inner_exclusive_access(file!(), line!());
    let process = task.process.upgrade().unwrap();
    let tid = task_inner.res.as_ref().unwrap().tid;
    // record exit code
    task_inner.exit_code = Some(exit_code);
    task_inner.res = None;
    let task_info = TaskInfo {
        pid: task.get_pid(),
        ppid: task.get_ppid(),
        status: TaskStatus::Dead,
        user_time: task_inner.user_time,
        kernel_time: task_inner.kernel_time,
        time_created: task_inner.time_created,
        first_time: task_inner.first_time,
    };
    write_proc(task_info);
    // here we do not remove the thread since we are still using the kstack
    // it will be deallocated when sys_waittid is called
    drop(task_inner);
    drop(task);
    // however, if this is the main thread of current process
    // the process should terminate at once
    if tid == 0 {
        let pid = process.getpid();
        if pid == IDLE_PID {
            println!(
                "[kernel] Idle process exit with exit_code {} ...",
                exit_code
            );
            if exit_code != 0 {
                //crate::sbi::shutdown(255); //255 == -1 for err hint
                shutdown(true);
            } else {
                //crate::sbi::shutdown(0); //0 for success hint
                shutdown(false);
            }
        }
        remove_from_pid2process(pid);
        let mut process_inner = process.inner_exclusive_access(file!(), line!());
        // mark this process as a zombie process
        process_inner.is_zombie = true;
        // record exit code of main process
        process_inner.exit_code = exit_code;

        {
            // move all child processes under init process
            let mut initproc_inner = INITPROC.inner_exclusive_access(file!(), line!());
            for child in process_inner.children.iter() {
                child.inner_exclusive_access(file!(), line!()).parent =
                    Some(Arc::downgrade(&INITPROC));
                initproc_inner.children.push(child.clone());
            }
        }

        // deallocate user res (including tid/trap_cx/ustack) of all threads
        // it has to be done before we dealloc the whole memory_set
        // otherwise they will be deallocated twice
        let mut recycle_res = Vec::<TaskUserRes>::new();
        for task in process_inner.tasks.iter().filter(|t| t.is_some()) {
            let task = task.as_ref().unwrap();
            let mut task_inner = task.inner_exclusive_access(file!(), line!());
            if let Some(res) = task_inner.res.take() {
                recycle_res.push(res);
            }
        }
        // dealloc_tid and dealloc_user_res require access to PCB inner, so we
        // need to collect those user res first, then release process_inner
        // for now to avoid deadlock/double borrow problem.
        drop(process_inner);
        recycle_res.clear();

        let mut process_inner = process.inner_exclusive_access(file!(), line!());
        process_inner.children.clear();
        // deallocate other data in user space i.e. program code/data section
        process_inner.memory_set.recycle_data_pages();
        // drop file descriptors
        process_inner.fd_table.clear();
        // Remove all tasks except for the main thread itself.
        // This is because we are still using the kstack under the TCB
        // of the main thread. This TCB, including its kstack, will be
        // deallocated when the process is reaped via waitpid.
        while process_inner.tasks.len() > 1 {
            process_inner.tasks.pop();
        }
    }
    drop(process);
    // we do not have to save task context
    let mut _unused = TaskContext::zero_init();
    schedule(&mut _unused as *mut _);
}

lazy_static! {
    pub static ref INITPROC: Arc<ProcessControlBlock> = {
        let inode = open_file(ROOT_INODE.clone(), "initproc", OpenFlags::RDONLY).unwrap();
        let v = inode.read_all();
        ProcessControlBlock::new(v.as_slice(), ROOT_INODE.clone(), "initproc")
    };
}

pub fn add_initproc() {
    let _initproc = INITPROC.clone();
}

pub fn check_signals_of_current() -> Option<(i32, &'static str)> {
    let process = current_process();
    let process_inner = process.inner_exclusive_access(file!(), line!());
    process_inner.signals.check_error()
}

pub fn current_add_signal(signal: SignalFlags) {
    let process = current_process();
    let mut process_inner = process.inner_exclusive_access(file!(), line!());
    process_inner.signals |= signal;
}
