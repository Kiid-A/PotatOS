#![no_std]
#![no_main]

extern crate user_lib;

use user_lib::{exec, fork, println, wait, yield_};

#[no_mangle]
fn main() -> i32 {
    if fork() == 0 {
        exec("user_shell\0", &[core::ptr::null::<u8>()]);
    } else {
        loop {
            let mut exit_code: i32 = 0;
            let pid = wait(&mut exit_code);
            if pid == -1 {
                println!("[initproc] yield and wait again...");
                yield_();
                continue;
            }
            /*
            println!(
                "[initproc] Released a zombie process, pid={}, exit_code={}",
                pid,
                exit_code,
            );
            */
            if pid == -10 {
                println!("[initproc] All tasks have exited, shutting down...");
                break;
            }
            else {
                println!(
                    "[initproc] Released a zombie process, pid={}, exit_code={}",
                    pid,
                    exit_code,
                );
            }
        }
    }
    0
}
