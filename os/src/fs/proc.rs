use spin::Mutex;
use crate::task::{self, TaskInfo};
use alloc::{string::String, sync::Arc};
use lazy_static::*;

// TODO: change /proc into vectors to reduce memory usage

const MAX_TASKS: usize = 100;

lazy_static! {
    static ref PROCESS_INFO_VEC: Arc<Mutex<[TaskInfo]>> = Arc::new(Mutex::new([TaskInfo::default(); MAX_TASKS]));
}

pub fn init_proc() -> isize {
    for ti in PROCESS_INFO_VEC.lock().iter_mut() {
        *ti = TaskInfo::default();
    }
    0
}

pub fn read_proc(pid: usize, task_info: *mut TaskInfo) -> isize {
    let ti = PROCESS_INFO_VEC.lock()[pid];
    if ti.pid != pid {
        return -1;
    }

    unsafe { *task_info = ti };
    0
}

// pub fn read_all(task_infos: &mut [TaskInfo]) -> isize {
//     let ti = PROCESS_INFO_VEC.lock();
//     for i in 0..MAX_TASKS {
//         task_infos[i] = ti[i];
//     }
//     0
// }


pub fn write_proc(task_info: TaskInfo) -> isize {
    // if task_info.pid <= 1 {
    //     return 0;
    // }
    // let filename = task_info.pid.to_string();
    // info!("write proc: {}", filename);
    // let proc_inode = PROC_INODE.clone();
    
    // info!("filename: {}", filename);
    // let inode = proc_inode.find(&filename);
    // if inode.is_none() {
    //     info!("create file");
    //     proc_inode.create_file(&filename);
    // }
    // info!("find file");

    // let inode = inode.unwrap();
    // let bytes: &[u8] = unsafe {
    //     core::slice::from_raw_parts(
    //         &task_info as *const TaskInfo as *const u8,
    //         core::mem::size_of::<TaskInfo>(),
    //     )
    // }; 
    // info!("ready to write proc");
    // inode.write_at(0, bytes);
    // info!("write proc done, write {} bytes", bytes.len());
    // let ti = TaskInfo::default();
    // read_proc(1, &ti as *const TaskInfo as *mut TaskInfo);
    // println!("proc: pid: {}, ppid: {}, status: {:?}, user_time: {}, kernel_time: {}, time_created: {}",
    //     ti.pid, ti.ppid, ti.status, ti.user_time, ti.kernel_time, ti.time_created);
    if task_info.pid >= MAX_TASKS {
        println!("pid out of range {}", task_info.pid);
        return -1;
    }
    PROCESS_INFO_VEC.lock()[task_info.pid] = task_info;
    0
}

pub fn remove_proc(pid: usize) -> isize {
    PROCESS_INFO_VEC.lock()[pid] = TaskInfo::default();
    0
}