#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;

use user_lib::{close, fstat, mkdir, open, OpenFlags, Stat};

#[no_mangle]
pub fn main(argc: usize, argv: &[&str]) -> isize {
    if argc != 2 {
        println!("Usage: touch <file>");
        return -1;
    }

    let fd = open(argv[1], OpenFlags::CREATE | OpenFlags::RDONLY);
    close(fd as usize);
    0    
}