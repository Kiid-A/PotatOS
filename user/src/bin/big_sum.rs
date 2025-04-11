#![no_std]
#![no_main]

#![macro_use]

use oorandom::Rand64;
use user_lib::{exec, exit, fork, println, thread_create, wait};

const M: u64 = 2022210109;
const N: u64 = 1e7 as u64;

