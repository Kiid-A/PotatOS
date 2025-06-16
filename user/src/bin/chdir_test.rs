#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;
use user_lib::{chdir, close, exec, fstat, getcwd, link, mkdir, open, read, unlink, write, OpenFlags, Stat};


#[no_mangle]
pub fn main(argc: usize, argv: &[&str]) -> i32 {

    let dir = "test_dir\0";
    mkdir(dir);
    let mut buffer = [0u8; 1024]; // 1KiB
    let len = getcwd(&mut buffer);
    if len < 0 {
        println!("get cwd failed!");
        return -1;
    }
    let cwd = core::str::from_utf8(&buffer[..len as usize]).unwrap();
    println!("cwd: {}", cwd);

    chdir(dir);
    let mut buffer = [0u8; 1024]; // 1KiB
    let len = getcwd(&mut buffer);
    if len < 0 {
        println!("get cwd failed!");
        return -1;
    }
    let cwd = core::str::from_utf8(&buffer[..len as usize]).unwrap();
    println!("cwd: {}", cwd);
    0
}