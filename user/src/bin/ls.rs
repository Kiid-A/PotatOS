#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;

use user_lib::{open, OpenFlags};

#[no_mangle]
pub fn main(argc: usize, argv: &[&str]) -> isize {
    panic!("ls: not implemented");
}