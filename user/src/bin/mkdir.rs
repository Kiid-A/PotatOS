#![no_std]
#![no_main]

use user_lib::{fstat, mkdir, open, OpenFlags, Stat, StatMode};

#[macro_use]
extern crate user_lib;

#[no_mangle]
pub fn main() -> i32 {
    let name = "test\0";
    
    let rc = mkdir(name);
    if rc < 0 {
        println!("Failed to mkdir");
        return -1;
    }
    let fd = open(name, OpenFlags::RDONLY);
    assert!(fd > 0);
    let stat = Stat::new(); 
    let ret = fstat(fd as usize, &stat);
    assert!(ret == 0);
    println!("stat mode: {}", stat.mode.bits());
    0
}