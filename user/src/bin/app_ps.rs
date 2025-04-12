#![no_std]
#![no_main]

use user_lib::{exec, fork, read_proc, sleep, wait4, TaskInfo, TaskStatus, WaitOption};
extern crate alloc;
use alloc::vec::Vec;


#[macro_use]
extern crate user_lib;

fn clear_screen() {
    print!("\x1B[2J\x1B[1;1H");
}

fn read_all() -> Vec<TaskInfo> {
    let mut v: Vec<TaskInfo> = Vec::new();
    for i in 0..512 {
        let mut info = TaskInfo::new();
        let ret = read_proc(i, &mut info);
        if ret == 0 && info.status != TaskStatus::Dead {
            v.push(info);
        }
    }
    v
}

#[no_mangle]
pub fn main(argc: usize, argv: &[&str]) -> isize {
    if argc != 2 {
        println!("Usage: gui_ps <app_name>");
        return 0;
    }
    let pid = fork();
    if pid == 0 {
        exec(argv[1], &[core::ptr::null::<u8>()]);
    } else {
        let mut exit_code: i32 = pid as i32 + 1;
        println!("child pid: {}", pid);
        loop {
            // sleep(2 * 1000);
            wait4(pid, &mut exit_code, WaitOption::WNOHANG.bits());
            if exit_code == 0 {
                break;
            }
            let v = read_all();
            for ti in v {
                let status = match ti.status {
                    TaskStatus::Ready => "Ready",
                    TaskStatus::Running => "Running",
                    TaskStatus::Blocked => "Blocked",
                    TaskStatus::Dead => "Dead",
                };
                println!("pid: {} ppid: {} status: {} user_time: {} kernel_time: {} time_created: {} last_time: {}", 
                            ti.pid, ti.ppid, status, ti.user_time, ti.kernel_time, ti.time_created, ti.first_time);
            }
            sleep(2 * 250);
            clear_screen();
        } 
    }   
    0
}