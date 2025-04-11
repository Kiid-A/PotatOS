#![no_std]
#![no_main]

use riscv::register::hgatp::read;
use user_lib::{read_proc, TaskInfo, TaskStatus};

#[macro_use]
extern crate user_lib;

#[no_mangle]
pub fn main(argc: usize, argv: &[&str]) -> i32 {
    if argc != 2 {
        println!("Usage: readproc <pid>, num: {}", argc);
        return -1;
    }
    let ti = TaskInfo::new();
    let pid = argv[1].parse::<usize>().unwrap();
    let flag = read_proc(pid, &ti);
    if flag == -1 {
        println!("this process has not been executed yet");
        return 0;
    } else if flag == -2 {
        println!("this process has exited");
        return 0;
    }
    let status = match ti.status {
        TaskStatus::Ready => "Ready",
        TaskStatus::Running => "Running",
        TaskStatus::Blocked => "Blocked",
        TaskStatus::Dead => "Dead",
    };
    println!("pid: {} ppid: {} status: {} user_time: {} kernel_time: {} time_created: {} first_time: {}", 
            ti.pid, ti.ppid, status, ti.user_time, ti.kernel_time, ti.time_created, ti.first_time);
    0
}