#![no_std]
#![no_main]

use user_lib::{close, fstat, mkdir, open, OpenFlags, Stat, StatMode};

#[macro_use]
extern crate user_lib;

#[no_mangle]
pub fn main(argc: usize, argv: &[&str]) -> i32 {
    if argc != 2 {
        println!("Usage: mkdir <dirname>");
        return -1;
    }

    mkdir(argv[1]) as i32
}