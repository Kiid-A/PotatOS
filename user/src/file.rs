use super::*;

bitflags! {
    pub struct OpenFlags: u32 {
        const RDONLY = 0;
        const WRONLY = 1 << 0;
        const RDWR = 1 << 1;
        const CREATE = 1 << 9;
        const TRUNC = 1 << 10;
    }
}

pub fn dup(fd: usize) -> isize {
    sys_dup(fd)
}
pub fn open(path: &str, flags: OpenFlags) -> isize {
    sys_open(path, flags.bits)
}
pub fn close(fd: usize) -> isize {
    sys_close(fd)
}
pub fn pipe(pipe_fd: &mut [usize]) -> isize {
    sys_pipe(pipe_fd)
}
pub fn read(fd: usize, buf: &mut [u8]) -> isize {
    sys_read(fd, buf)
}
pub fn write(fd: usize, buf: &[u8]) -> isize {
    sys_write(fd, buf)
}

pub fn getcwd(buf: &mut [u8]) -> isize {
    sys_getcwd(buf, buf.len())
}

pub fn chdir(path: &str) -> isize {
    sys_chdir(path)
}

pub fn mkdir(path: &str) -> isize {
    sys_mkdir(path)
}

pub fn fstat(fd: usize, st: &Stat) -> isize {
    sys_fstat(fd, st)
}

pub fn task_info(ti: &TaskInfo) -> isize {
    sys_task_info(ti)
}

pub fn read_proc(pid: usize, ti: &TaskInfo) -> isize {
    sys_read_proc(pid, ti)
}

pub fn ls() -> isize {
    sys_ls()
}

pub fn link(old_path: &str, new_path: &str) -> isize {
    sys_linkat(old_path, new_path)
}

pub fn unlink(path: &str) -> isize {
    sys_unlinkat(path)
}

pub fn remove(path: &str, args: &str) -> isize {
    sys_remove(path, args)
}