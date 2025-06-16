bitflags! {
    pub struct StatMode: u32 {
        const NULL  = 0;
        /// directory
        const DIR   = 0o040000;
        /// ordinary regular file
        const FILE  = 0o100000;
    }
}

#[repr(C)]
#[derive(Debug)]
pub struct Stat {
    pub dev: u64,
    pub ino: u64,
    pub mode: StatMode,
    pub nlink: u32,
    pad: [u64; 7],
}

impl Stat {
    pub fn new(inode_: u64, mode_: StatMode, nlink_: u32) -> Self {
        Self {
            dev: 0,
            ino: inode_,
            mode: mode_,
            nlink: nlink_,
            pad: [0; 7],
        }
    }
}
