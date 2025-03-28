#![no_std]
#![no_main]

use user_lib::{close, fstat, open, OpenFlags, Stat, StatMode};

#[macro_use]
extern crate user_lib;

#[no_mangle]
pub fn main(argc: usize, argv: &[&str]) -> i32 {
    if argc != 2 {
        println!("Usage: fstat <file>");
        return -1;
    }
    let fd = open(argv[1], OpenFlags::WRONLY);
    assert!(fd > 0);
    let fd = fd as usize;
    let stat = Stat::new();
    fstat(fd, &stat);
    println!("find file: {}", fd);
    let mode = if stat.mode.clone() == StatMode::FILE {
        "file"
    } else {
        "directory"
    };
    println!("dev: {}, inode: {}, mode: {}, nlinks: {}", stat.dev, stat.ino, mode, stat.nlink);
    0
    // let fname = "cat\0";
    // // let fd = open(fname, OpenFlags::CREATE | OpenFlags::WRONLY);
    // let fd = open(fname, OpenFlags::WRONLY);
    // assert!(fd > 0);
    // let fd = fd as usize;
    // let stat: Stat = Stat::new();
    // let ret = fstat(fd, &stat);
    // assert_eq!(ret, 0);
    // assert_eq!(stat.mode, StatMode::DIR);
    // assert_eq!(stat.nlink, 1);
    // close(fd);
    // // unlink(fname);
    // // It's recommended to rebuild the disk image. This program will not clean the file "fname1".
    // println!("Test fstat OK!");
    // 0
}