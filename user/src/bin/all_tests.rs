#![no_std]
#![no_main]

extern crate user_lib;

static ALL_TESTS: &[(&str, i32)] = &[
    ("fs_test\0", 0),
    ("mm_test\0", 0),
    ("sync_sem_test\0", 0),
    ("task_test\0", 0),
];

use user_lib::{exec, fork, waitpid, println};

fn run_all_tests(tests: &[(&str, i32)]) -> i32 {
    let mut pass_num = 0;
    for test in tests {
        println!("AllTests: Running {}", test.0);
        let pid = fork();
        if pid == 0 {
            exec(test.0, &[core::ptr::null::<u8>()]);
            panic!("unreachable!");
        } else {
            let mut exit_code: i32 = 0;
            let wait_pid = waitpid(pid as usize, &mut exit_code);
            assert_eq!(pid, wait_pid);
            if exit_code == test.1 {
                pass_num += 1;
            }
            println!(
                "\x1b[32mAllTests: Test {} in Process {} exited with code {}\x1b[0m",
                test.0, pid, exit_code
            );
        }
    }
    pass_num
}

#[no_mangle]
pub fn main() -> i32 {
    let succ_num = run_all_tests(ALL_TESTS);
    if succ_num == ALL_TESTS.len() as i32 {
        println!("All {} tests passed! all_tests passed!", ALL_TESTS.len());
        0
    } else {
        println!(
            "all_tests: {} of {} tests passed, failed!",
            succ_num,
            ALL_TESTS.len()
        );
        -1
    }
}