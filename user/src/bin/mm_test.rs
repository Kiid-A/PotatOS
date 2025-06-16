#![no_std]
#![no_main]


use user_lib::{println};

extern crate user_lib;


#[no_mangle]
pub fn main() -> i32 {
    // 堆分配测试
    {
        let mut v = [0usize; 100];
        for i in 0..v.len() {
            v[i] = i;
        }
        for i in 0..v.len() {
            assert!(v[i] == i);
        }
        println!("heap basic test passed");
    }

    // 大量堆分配测试
    {
        let mut sum = 0usize;
        for i in 0..1000 {
            sum += i;
        }
        assert!(sum == (0..1000).sum());
        println!("heap stress test passed");
    }

    // 简单栈分配测试
    {
        fn stack_test(n: usize) -> usize {
            if n == 0 { 1 } else { n * stack_test(n - 1) }
        }
        assert!(stack_test(5) == 120);
        println!("stack test passed");
    }

    // 简单内存写读测试
    {
        let mut arr = [0u8; 32];
        for i in 0..arr.len() {
            arr[i] = i as u8;
        }
        for i in 0..arr.len() {
            assert!(arr[i] == i as u8);
        }
        println!("memory rw test passed");
    }

    println!("mm_test passed!");
    0
}