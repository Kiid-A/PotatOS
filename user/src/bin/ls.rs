#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;

use user_lib::{fstat, mkdir, open, OpenFlags, Stat};

// no arg: ls .
// or: ls argv
#[no_mangle]
pub fn main(argc: usize, argv: &[&str]) -> isize {
    panic!("ls not implemeneted");

    // let name = "test\0";
    
    // let rc = mkdir(name);
    // if rc < 0 {
    //     println!("Failed to mkdir");
    //     return -1;
    // }
    // let fd = open(name, OpenFlags::RDONLY);
    // assert!(fd > 0);
    // let stat = Stat::new(); 
    // let ret = fstat(fd as usize, &stat);
    // assert!(ret == 0);
    // println!("stat mode: {}", stat.mode.bits());
    
    // let rs = ls
    // 0    
}