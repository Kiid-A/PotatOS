use super::id::TaskUserRes;
use super::{kstack_alloc, process, KernelStack, ProcessControlBlock, TaskContext};
use crate::timer::{get_time, get_time_ms};
use crate::trap::TrapContext;
use crate::{
    mm::PhysPageNum,
    sync::{UPIntrFreeCell, UPIntrRefMut},
};
use alloc::string::String;
use alloc::sync::{Arc, Weak};

pub struct TaskControlBlock {
    pub pid: usize,
    pub ppid: usize,
    // immutable
    pub process: Weak<ProcessControlBlock>,
    pub kstack: KernelStack,
    // mutable
    pub inner: UPIntrFreeCell<TaskControlBlockInner>,
}

impl TaskControlBlock {
    pub fn inner_exclusive_access(&self, file: &'static str, line: u32,) -> UPIntrRefMut<'_, TaskControlBlockInner> {
        self.inner.exclusive_access(file, line)
    }

    pub fn get_user_token(&self) -> usize {
        let process = self.process.upgrade().unwrap();
        let inner = process.inner_exclusive_access(file!(), line!());
        inner.memory_set.token()
    }

    pub fn get_pid(&self) -> usize {
        self.pid
    }

    pub fn get_ppid(&self) -> usize {
        self.ppid
    }
}

pub struct TaskControlBlockInner {
    pub res: Option<TaskUserRes>,
    pub trap_cx_ppn: PhysPageNum,
    pub task_cx: TaskContext,
    pub task_status: TaskStatus,
    pub exit_code: Option<i32>,

    pub user_time: usize,
    pub kernel_time: usize,
    pub time_created: usize,    
    pub first_time: usize,
    pub stop_watch: usize,
}

impl TaskControlBlockInner {
    pub fn get_trap_cx(&self) -> &'static mut TrapContext {
        self.trap_cx_ppn.get_mut()
    }

    #[allow(unused)]
    fn get_status(&self) -> TaskStatus {
        self.task_status
    }

    pub fn refresh_watch(&mut self) -> usize {
        let current = self.stop_watch;
        self.stop_watch = get_time_ms();
        self.stop_watch - current
    }

    pub fn user_time_start(&mut self) {
        self.kernel_time += self.refresh_watch()
    }

    pub fn user_time_end(&mut self) {
        self.user_time += self.refresh_watch()
    }

    // pub fn kernel_time_start(&mut self) {
    //     self.refresh_watch();
    // }

    // pub fn kernel_time_end(&mut self) {
    //     self.kernel_time += self.refresh_watch();
    // }
}

impl TaskControlBlock {
    pub fn new(
        process: Arc<ProcessControlBlock>,
        ustack_base: usize,
        alloc_user_res: bool,
    ) -> Self {
        let res = TaskUserRes::new(Arc::clone(&process), ustack_base, alloc_user_res);
        let trap_cx_ppn = res.trap_cx_ppn();
        let kstack = kstack_alloc();
        let kstack_top = kstack.get_top();
        Self {
            pid: process.getpid(),
            ppid: process.ppid,
            process: Arc::downgrade(&process),
            kstack,
            inner: unsafe {
                UPIntrFreeCell::new(TaskControlBlockInner {
                    res: Some(res),
                    trap_cx_ppn,
                    task_cx: TaskContext::goto_trap_return(kstack_top),
                    task_status: TaskStatus::Ready,
                    exit_code: None,
                    user_time: 0,
                    kernel_time: 0,
                    first_time: 0,
                    time_created: get_time_ms(),
                    stop_watch: 0,
                })
            },
            
        }
    }
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum TaskStatus {
    Ready,
    Running,
    Blocked,
    Dead,
}

#[derive(Debug, Clone, Copy)]
pub struct TaskInfo {
    pub pid: usize,
    pub ppid: usize,
    pub status: TaskStatus,

    pub user_time: usize,
    pub kernel_time: usize,
    pub time_created: usize,
    pub first_time: usize,
}

impl TaskInfo {
    pub fn default() -> Self {
        Self {
            pid: 0,
            ppid: 0,
            status: TaskStatus::Running,
            user_time: 0,
            kernel_time: 0,
            time_created: 0,
            first_time: 0,
        }
    }
}