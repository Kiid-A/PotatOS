pub mod fstat;
pub mod inode;
mod pipe;
pub mod proc;
mod stdio;

use crate::mm::UserBuffer;
use crate::task;

pub trait File: Send + Sync {
    fn readable(&self) -> bool;
    fn writable(&self) -> bool;
    fn read(&self, buf: UserBuffer) -> usize;
    fn write(&self, buf: UserBuffer) -> usize;
    fn stat(&self) -> Stat;
}

pub use fstat::Stat;
use fstat::StatMode;
pub use inode::list_apps;
pub use inode::ROOT_INODE;
pub use inode::{open_file, OpenFlags};
pub use pipe::make_pipe;
pub use stdio::{Stdin, Stdout};
use task::{TaskControlBlock, TaskInfo};
