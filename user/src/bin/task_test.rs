#![no_std]
#![no_main]

extern crate user_lib;
use user_lib::{fork, getpid, wait, yield_, exit, sleep, println};

fn fork_test() {
    let pid = fork();
    if pid == 0 {
        // child
        println!("[fork_test] child pid = {}", getpid());
        for _ in 0..3 {
            println!("[fork_test] child yield");
            yield_();
        }
        println!("[fork_test] child exit");
        exit(123);
    } else {
        // parent
        println!("[fork_test] parent pid = {}, child pid = {}", getpid(), pid);
        let mut code: i32 = 0;
        let wait_pid = wait(&mut code);
        println!("[fork_test] parent wait child {} exit code {}", wait_pid, code);
        assert!(wait_pid == pid && code == 123, "fork/wait test failed");
    }
}

fn multi_fork_test() {
    let mut children = [0isize; 3];
    for i in 0..3 {
        let pid = fork();
        if pid == 0 {
            println!("[multi_fork_test] child {} running", i);
            sleep(50 * (i + 1));
            println!("[multi_fork_test] child {} exiting", i);
            exit(10 + i as i32);
        } else {
            children[i] = pid;
        }
    }
    let mut exit_codes = [0i32; 3];
    for i in 0..3 {
        let pid = wait(&mut exit_codes[i]);
        println!("[multi_fork_test] parent waited child {} exit code {}", pid, exit_codes[i]);
        assert!(children.contains(&pid), "waited unknown child");
    }
    println!("multi_fork_test passed");
}

#[no_mangle]
pub fn main() -> i32 {
    fork_test();
    multi_fork_test();
    println!("task/process test passed!");
    0
}