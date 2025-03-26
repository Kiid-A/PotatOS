#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;
use user_lib::get_cwd;

#[no_mangle]
pub fn main() -> i32 {
     let mut buffer = [0u8; 1024]; // 1KiB
     let len = get_cwd(&mut buffer);
     if len < 0 {
         println!("get cwd failed!");
         return -1;
     }
     let cwd = core::str::from_utf8(&buffer[..len as usize]).unwrap();
     println!("cwd: {}", cwd);
     0
}