pub mod inode;
mod pipe;
mod stdio;
pub mod fstat;

use crate::mm::UserBuffer;

pub trait File: Send + Sync {
    fn readable(&self) -> bool;
    fn writable(&self) -> bool;
    fn read(&self, buf: UserBuffer) -> usize;
    fn write(&self, buf: UserBuffer) -> usize;
    fn stat(&self) -> Stat;
}

use fstat::StatMode;
pub use inode::ROOT_INODE;
pub use inode::{open_file, OpenFlags};
pub use pipe::make_pipe;
pub use stdio::{Stdin, Stdout};
pub use fstat::Stat;
pub use inode::list_apps;