#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;

use user_lib::{fstat, mkdir, open, OpenFlags, Stat};

// no arg: ls .
// or: ls argv
#[no_mangle]
pub fn main() -> isize {
    panic!("ls not implemeneted");
    
}