#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;
extern crate alloc;

use user_lib::{close, open, read, OpenFlags};

#[no_mangle]
pub fn main(argc: usize, argv: &[&str]) -> i32 {
    println!("argc = {}", argc);
    for (i, arg) in argv.iter().enumerate() {
        println!("argv[{}] = {}", i, arg);
    }
    assert!(argc == 2);
    let fd = open(argv[1], OpenFlags::RDONLY);
    if fd == -1 {
        panic!("Error occurred when opening file");
    }
    let fd = fd as usize;
    let mut buf = [0u8; 4096];
    loop {
        let size = read(fd, &mut buf) as usize;
        if size == 0 {
            break;
        }
        // print!("{}", core::str::from_utf8(&buf[..size]).unwrap());
        match core::str::from_utf8(&buf[..size]) {
            Ok(s) => println!("{}", s),
            Err(e) => {
                println!("Error converting to UTF-8: {}", e);
                for byte in &buf[..size] {
                    print!("{:02x} ", byte);
                }
                println!("");
            }
        }
    }
    close(fd);
    0
}
