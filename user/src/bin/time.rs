#![no_std]
#![no_main]

use user_lib::{exec, fork, get_time, wait4, WaitOption};

#[macro_use]
extern crate user_lib;
extern crate alloc;

#[no_mangle]
fn main(argc: usize, argv: &[&str]) -> i32 {
    if argc != 2 {
        println!("Usage: time <app_name>");
        return 0;
    }

    let pid = fork();
    let start_time = get_time();
    if pid == 0 {
        exec(argv[1], &[core::ptr::null::<u8>()]);
    } else {
        let mut exit_code: i32 = pid as i32 + 1;

        loop {
            wait4(pid, &mut exit_code, WaitOption::WNOHANG.bits());
            if exit_code == 0 {
                break;
            }
        }
    }

    println!("{} finished in {} ms", argv[1], get_time() - start_time);
    0
}
