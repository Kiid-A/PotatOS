use crate::fs::proc::{self, read_proc};
use crate::fs::{open_file, OpenFlags};
use crate::mm::{translated_ref, translated_refmut, translated_str};
use crate::task::{
    current_process, current_task, current_user_token, exit_current_and_run_next, pid2process,
    suspend_current_and_run_next, SignalFlags, TaskInfo,
};
use crate::timer::get_time_ms;
use alloc::string::String;
use alloc::sync::Arc;
use alloc::vec::Vec;

bitflags! {
    struct WaitOption: u32 {
        const WNOHANG    = 1;
        const WUNTRACED  = 2;
        const WEXITED    = 4;
        const WCONTINUED = 8;
        const WNOWAIT    = 0x1000000;
    }
}

pub fn sys_exit(exit_code: i32) -> ! {
    exit_current_and_run_next(exit_code);
    panic!("Unreachable in sys_exit!");
}

pub fn sys_yield() -> isize {
    suspend_current_and_run_next();
    0
}

pub fn sys_get_time() -> isize {
    get_time_ms() as isize
}

pub fn sys_getpid() -> isize {
    current_task().unwrap().process.upgrade().unwrap().getpid() as isize
}

pub fn sys_fork() -> isize {
    let current_process = current_process();
    let new_process = current_process.fork();
    let new_pid = new_process.getpid();
    // modify trap context of new_task, because it returns immediately after switching
    let new_process_inner = new_process.inner_exclusive_access(file!(), line!());
    let task = new_process_inner.tasks[0].as_ref().unwrap();
    let trap_cx = task.inner_exclusive_access(file!(), line!()).get_trap_cx();
    // we do not have to move to next instruction since we have done it before
    // for child process, fork returns 0
    trap_cx.x[10] = 0;
    new_pid as isize
}

pub fn sys_exec(path: *const u8, mut args: *const usize) -> isize {
    let token = current_user_token();
    let path = translated_str(token, path);
    let mut args_vec: Vec<String> = Vec::new();
    loop {
        let arg_str_ptr = *translated_ref(token, args);
        if arg_str_ptr == 0 {
            break;
        }
        args_vec.push(translated_str(token, arg_str_ptr as *const u8));
        unsafe {
            args = args.add(1);
        }
    }
    let process = current_process();
    // println!("current pid: {}", process.clone().pid.0);
    let cwd = process.inner_exclusive_access(file!(), line!()).cwd.clone();
    if let Some(app_inode) = open_file(cwd.clone(), path.as_str(), OpenFlags::RDONLY) {
        let all_data = app_inode.read_all();
        let argc = args_vec.len();
        process.exec(all_data.as_slice(), args_vec);
        // return argc because cx.x[10] will be covered with it later
        argc as isize
    } else {
        -1
    }
}

/// If there is not a child process whose pid is same as given, return -1.
/// Else if there is a child process but it is still running, return -2.
pub fn sys_waitpid(pid: isize, exit_code_ptr: *mut i32, option: u32) -> isize {
    let process = current_process();
    let option = WaitOption::from_bits(option).unwrap();
    // find a child process
    loop {
        let mut inner = process.inner_exclusive_access(file!(), line!());
        if !inner
            .children
            .iter()
            .any(|p| pid == -1 || pid as usize == p.getpid())
        {
            drop(inner);
            return -1;
            // ---- release current PCB
        }
        let pair = inner.children.iter().enumerate().find(|(_, p)| {
            // ++++ temporarily access child PCB exclusively
            let inner = p.inner_exclusive_access(file!(), line!());
            let flag = inner.is_zombie && (pid == -1 || pid as usize == p.getpid());
            drop(inner);
            return flag;
            // ++++ release child PCB
        });
        if let Some((idx, _)) = pair {
            let child = inner.children.remove(idx);
            // confirm that child will be deallocated after being removed from children list
            assert_eq!(Arc::strong_count(&child), 1);
            let found_pid = child.getpid();
            // ++++ temporarily access child PCB exclusively
            let exit_code = child.inner_exclusive_access(file!(), line!()).exit_code;
            // ++++ release child PCB
            *translated_refmut(inner.memory_set.token(), exit_code_ptr) = exit_code;
            return found_pid as isize;
        } else {
            drop(inner);
            drop(process);
            if option.contains(WaitOption::WNOHANG) {
                return 0;
            }
            return -2;
        }
    }
    // ---- release current PCB automatically
}

pub fn sys_kill(pid: usize, signal: u32) -> isize {
    if let Some(process) = pid2process(pid) {
        if let Some(flag) = SignalFlags::from_bits(signal) {
            process.inner_exclusive_access(file!(), line!()).signals |= flag;
            0
        } else {
            -1
        }
    } else {
        -1
    }
}

pub fn sys_task_info(task_info_addr: usize) -> isize {
    let task = current_task().unwrap();
    let token = current_user_token();
    let task_info = translated_refmut(token, task_info_addr as *mut TaskInfo);
    let inner = task.inner.exclusive_access(file!(), line!());
    let ti = TaskInfo {
        pid: task.get_pid(),
        ppid: task.get_ppid(),
        status: inner.task_status,
        user_time: inner.user_time,
        kernel_time: inner.kernel_time,
        time_created: inner.time_created,
        first_time: inner.first_time,
    };
    *task_info = ti;
    0
}

pub fn sys_read_proc(pid: usize, ti_addr: usize) -> isize {
    let process = pid2process(pid);
    // if process.is_none() || process.unwrap().inner_exclusive_access(file!(), line!()).is_zombie {
    //     return -2;
    // }
    if process.is_none() {
        return -1;
    }

    let token = current_user_token();
    let task_info = translated_refmut(token, ti_addr as *mut TaskInfo);
    read_proc(pid, task_info)
}
