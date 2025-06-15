#![no_std]
#![no_main]

use user_lib::{close, fstat, open, task_info, OpenFlags, StatMode, TaskInfo, TaskStatus};

#[macro_use]
extern crate user_lib;

#[no_mangle]
pub fn main(argc: usize, argv: &[&str]) -> i32 {
    if argc != 1 {
        println!("Usage: task_info <cmd>");
        return -1;
    }
    let ti = TaskInfo::new();
    task_info(&ti);
    let status = match ti.status {
        TaskStatus::Ready => "Ready",
        TaskStatus::Running => "Running",
        TaskStatus::Blocked => "Blocked",
        TaskStatus::Dead => "Dead",
    };
    println!("pid: {} ppid: {} status: {} user_time: {} kernel_time: {} time_created: {}", 
            ti.pid, ti.ppid, status, ti.user_time, ti.kernel_time, ti.time_created);
    0
}