#![no_std]
#![no_main]

#![macro_use]
extern crate user_lib;
extern crate alloc;

use alloc::vec::Vec;
use oorandom::Rand64;
use user_lib::{exec, exit, fork, println, thread_create, wait, waittid};

const T: u64 = 10;
const N: u64 = 1e8 as u64;
const M: u64 = 114514;


pub fn calc(n: u64) -> ! {
    let mut rng = Rand64::new(42);
    let mut sum = 0;
    let times = n + rng.rand_range(n/2..n);
    let size = rng.rand_range(100..123456);
    for _ in 0..times {
        let num: u64 = rng.rand_range(0..size);
        sum += num; 
        sum %= M;
    }
    println!("sum: {}", sum);
    
    exit(0);
}

#[no_mangle]
pub fn main() -> isize {
    // let mut v = Vec::new();
    // for _ in 0..T {
    //     v.push(thread_create(calc as usize, N as usize));
    // }
    for _ in 0..T {
        let pid = fork();
        if pid == 0 {
            calc(N);
        }
    }
    let mut exit_code: i32 = 0;
    for _ in 0..T {
        assert!(wait(&mut exit_code) > 0);
        assert_eq!(exit_code, 0);
    }
    // for tid in v.iter() {
    //     let exit_code = waittid(*tid as usize);
    //     println!("thread#{} exited with code {}", tid, exit_code);
    // }
    println!("large adder test passed!");
    0
}