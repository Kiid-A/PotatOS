#![no_std]
#![no_main]

use user_lib::{chdir, close, getcwd, mkdir, open, println, read, remove, sleep, unlink, write, OpenFlags, Stat};

extern crate user_lib;

#[no_mangle]
pub fn main() -> i32 {
    // 测试创建和写入文件
    let fname = "fs_test_file\0";
    let fd = open(fname, OpenFlags::CREATE | OpenFlags::WRONLY);
    assert!(fd >= 0, "open file failed");
    let content = b"hello, PotatOS fs!\0";
    let write_len = write(fd as usize, content);
    assert!(write_len == content.len() as isize, "write file failed");
    close(fd as usize);

    sleep(250);

    // 测试读取文件内容
    let fd = open(fname, OpenFlags::RDONLY);
    assert!(fd >= 0, "open file for read failed");
    let mut buf = [0u8; 32];
    let read_len = read(fd as usize, &mut buf);
    assert!(read_len == content.len() as isize, "read file failed");
    assert!(
        &buf[..(read_len as usize)] == content,
        "file content mismatch"
    );
    close(fd as usize);

    sleep(250);

    // 测试 stat 系统调用
    let fd = open(fname, OpenFlags::RDONLY);
    assert!(fd >= 0, "open file for stat failed");
    let mut stat = Stat::new();
    let ret = user_lib::fstat(fd as usize, &mut stat);
    assert!(ret == 0, "fstat failed");
    assert!(stat.mode.bits() & 0o100000 != 0, "not a regular file");
    close(fd as usize);

    sleep(250);

    // 测试删除文件
    let ret = remove(fname, "OP");
    assert!(ret == 0, "unlink file failed");

    sleep(250);

    // 测试删除不存在的文件
    let ret = remove("nonexistent_file\0", "OP");
    assert!(ret < 0, "unlink should fail for non-existent file");

    // 测试 mkdir + chdir
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

    sleep(250);

    chdir(dir);
    let mut buffer = [0u8; 1024]; // 1KiB
    let len = getcwd(&mut buffer);
    if len < 0 {
        println!("get cwd failed!");
        return -1;
    }
    let cwd = core::str::from_utf8(&buffer[..len as usize]).unwrap();
    println!("cwd: {}", cwd);

    chdir("/\0");

    println!("fs_test passed!");
    0
}
