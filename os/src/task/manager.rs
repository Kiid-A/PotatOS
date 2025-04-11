use super::{ProcessControlBlock, TaskControlBlock, TaskInfo, TaskStatus};
use crate::fs::proc::write_proc;
use crate::sync::UPIntrFreeCell;
use alloc::collections::{BTreeMap, VecDeque};
use alloc::sync::Arc;
use lazy_static::*;
use log::info;

pub struct TaskManager {
    ready_queue: VecDeque<Arc<TaskControlBlock>>,
}

/// A simple FIFO scheduler.
impl TaskManager {
    pub fn new() -> Self {
        Self {
            ready_queue: VecDeque::new(),
        }
    }
    pub fn add(&mut self, task: Arc<TaskControlBlock>) {
        self.ready_queue.push_back(task);
    }
    pub fn fetch(&mut self) -> Option<Arc<TaskControlBlock>> {
        self.ready_queue.pop_front()
    }
}

lazy_static! {
    pub static ref TASK_MANAGER: UPIntrFreeCell<TaskManager> =
        unsafe { UPIntrFreeCell::new(TaskManager::new()) };
    pub static ref PID2PCB: UPIntrFreeCell<BTreeMap<usize, Arc<ProcessControlBlock>>> =
        unsafe { UPIntrFreeCell::new(BTreeMap::new()) };
}

pub fn add_task(task: Arc<TaskControlBlock>) {
    TASK_MANAGER.exclusive_access(file!(), line!()).add(task);
}

pub fn wakeup_task(task: Arc<TaskControlBlock>) {
    let mut task_inner = task.inner_exclusive_access(file!(), line!());
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
    add_task(task);
    
    write_proc(task_info);
}

pub fn fetch_task() -> Option<Arc<TaskControlBlock>> {
    TASK_MANAGER.exclusive_access(file!(), line!()).fetch()
}

pub fn pid2process(pid: usize) -> Option<Arc<ProcessControlBlock>> {
    let map = PID2PCB.exclusive_access(file!(), line!());
    map.get(&pid).map(Arc::clone)
}

pub fn insert_into_pid2process(pid: usize, process: Arc<ProcessControlBlock>) {
    PID2PCB.exclusive_access(file!(), line!()).insert(pid, process);
}

pub fn remove_from_pid2process(pid: usize) {
    let mut map = PID2PCB.exclusive_access(file!(), line!());
    if map.remove(&pid).is_none() {
        panic!("cannot find pid {} in pid2task!", pid);
    }
}
