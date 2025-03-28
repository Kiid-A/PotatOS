#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;
use user_lib::{close, fstat, link, open, read, unlink, write, OpenFlags, Stat};

/// 测试 link/unlink，输出　Test link OK! 就算正确。

#[no_mangle]
pub fn main() -> i32 {
    unimplemented!()
}