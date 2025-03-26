use core::mem::size_of;

use crate::fs::{make_pipe, open_file, OpenFlags, Stat};
use crate::mm::{translated_byte_buffer, translated_refmut, translated_str, UserBuffer};
use crate::task::{current_process, current_user_token};
use alloc::string::String;
use alloc::sync::Arc;
use log::info;

use super::process;

pub fn sys_write(fd: usize, buf: *const u8, len: usize) -> isize {
    let token = current_user_token();
    let process = current_process();
    let inner = process.inner_exclusive_access();
    if fd >= inner.fd_table.len() {
        return -1;
    }
    if let Some(file) = &inner.fd_table[fd] {
        if !file.writable() {
            return -1;
        }
        let file = file.clone();
        // release current task TCB manually to avoid multi-borrow
        drop(inner);
        file.write(UserBuffer::new(translated_byte_buffer(token, buf, len))) as isize
    } else {
        -1
    }
}

pub fn sys_read(fd: usize, buf: *const u8, len: usize) -> isize {
    let token = current_user_token();
    let process = current_process();
    let inner = process.inner_exclusive_access();
    if fd >= inner.fd_table.len() {
        return -1;
    }
    if let Some(file) = &inner.fd_table[fd] {
        let file = file.clone();
        if !file.readable() {
            return -1;
        }
        // release current task TCB manually to avoid multi-borrow
        drop(inner);
        file.read(UserBuffer::new(translated_byte_buffer(token, buf, len))) as isize
    } else {
        -1
    }
}

pub fn sys_open(path: *const u8, flags: u32) -> isize {
    let process = current_process();
    let token = current_user_token();
    let path = translated_str(token, path);
    if let Some(inode) = open_file(path.as_str(), OpenFlags::from_bits(flags).unwrap()) {
        let mut inner = process.inner_exclusive_access();
        let fd = inner.alloc_fd();
        inner.fd_table[fd] = Some(inode);
        fd as isize
    } else {
        -1
    }
}

pub fn sys_close(fd: usize) -> isize {
    let process = current_process();
    let mut inner = process.inner_exclusive_access();
    if fd >= inner.fd_table.len() {
        return -1;
    }
    if inner.fd_table[fd].is_none() {
        return -1;
    }
    inner.fd_table[fd].take();
    0
}

pub fn sys_pipe(pipe: *mut usize) -> isize {
    let process = current_process();
    let token = current_user_token();
    let mut inner = process.inner_exclusive_access();
    let (pipe_read, pipe_write) = make_pipe();
    let read_fd = inner.alloc_fd();
    inner.fd_table[read_fd] = Some(pipe_read);
    let write_fd = inner.alloc_fd();
    inner.fd_table[write_fd] = Some(pipe_write);
    *translated_refmut(token, pipe) = read_fd;
    *translated_refmut(token, unsafe { pipe.add(1) }) = write_fd;
    0
}

pub fn sys_dup(fd: usize) -> isize {
    let process = current_process();
    let mut inner = process.inner_exclusive_access();
    if fd >= inner.fd_table.len() {
        return -1;
    }
    if inner.fd_table[fd].is_none() {
        return -1;
    }
    let new_fd = inner.alloc_fd();
    inner.fd_table[new_fd] = Some(Arc::clone(inner.fd_table[fd].as_ref().unwrap()));
    new_fd as isize
}

pub fn sys_get_cwd(buf: *mut u8, len: usize) -> isize {
    0
    // let process = current_process();
    // let inner = process.inner_exclusive_access();
    // let cwd = inner.cwd.clone();
    // let path = cwd.get_cwd();
    // let bytes = path.as_bytes();
    
    // if len < bytes.len() {
    //     return -1;
    // }

    // unsafe {
    //     let len = len.min(bytes.len());
    //     core::ptr::copy_nonoverlapping(bytes.as_ptr(), buf, len);
    // }

    // len as isize
}

pub fn sys_mkdir(pathname: *const u8) -> isize {
    let process = current_process();
    let token = current_user_token();
    let path = translated_str(token, pathname);
    let inner = process.inner_exclusive_access();
    let cwd = inner.cwd.clone();
    // drop(inner);
    info!("cwd inode number: {}", cwd.get_inode_id());
    let dir = cwd.create_dir(&path);
    if dir.is_none() {
        return -1;
    }
    0
}

pub fn sys_fstat(fd: usize, st_addr: usize) -> isize {
    let process = current_process();
    let token = current_user_token();
    let stat = translated_refmut(token, st_addr as *mut Stat);
    let inner = process.inner_exclusive_access();
    match inner.fd_table[fd] {
        Some(ref osinode) => {
            let new_st = osinode.stat();
            stat.ino = new_st.ino;
            stat.mode = new_st.mode;
            stat.nlink = new_st.nlink;
            return 0;
        }
        None => {
            return -1;
        }
    }
}

// TODO: link path shall be absolute ones. try to update it to support relative path 
pub fn sys_linkat(old_path_ptr: *const u8, new_path_ptr: *const u8) -> isize {
    let process = current_process();
    let token = current_user_token();
    let old_path = translated_str(token, old_path_ptr);
    let new_path = translated_str(token, new_path_ptr);
    let inner = process.inner_exclusive_access();
    let cwd = inner.cwd.clone();
    cwd.linkat(&old_path, &new_path)
}

pub fn sys_unlinkat(path_ptr: *const u8) -> isize {
    let process = current_process();
    let token = current_user_token();
    let path = translated_str(token, path_ptr);
    let inner = process.inner_exclusive_access();
    let cwd = inner.cwd.clone();
    cwd.unlinkat(&path) 
}