use core::cmp::min;
use core::mem::size_of;
use core::{clone, ptr};

use crate::fs::proc::read_proc;
use crate::fs::{make_pipe, open_file, proc, OpenFlags, Stat, ROOT_INODE};
use crate::mm::{translated_byte_buffer, translated_ref, translated_refmut, translated_str, UserBuffer};
use crate::task::{current_process, current_user_token, TaskInfo};
use alloc::string::String;
use alloc::sync::Arc;
use easy_fs::{DiskInodeType, Inode};
use log::info;

use super::process;

pub fn sys_write(fd: usize, buf: *const u8, len: usize) -> isize {
    let token = current_user_token();
    let process = current_process();
    let inner = process.inner_exclusive_access(file!(), line!());
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
    let inner = process.inner_exclusive_access(file!(), line!());
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
    let inner = process.inner_exclusive_access(file!(), line!());
    let cwd = inner.cwd.clone();
    drop(inner);
    if let Some(inode) = open_file(cwd.clone(), path.as_str(), OpenFlags::from_bits(flags).unwrap()) {
        let mut inner = process.inner_exclusive_access(file!(), line!());
        let fd = inner.alloc_fd();
        inner.fd_table[fd] = Some(inode);
        fd as isize
    } else {
        -1
    }
}

pub fn sys_close(fd: usize) -> isize {
    let process = current_process();
    let mut inner = process.inner_exclusive_access(file!(), line!());
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
    let mut inner = process.inner_exclusive_access(file!(), line!());
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
    let mut inner = process.inner_exclusive_access(file!(), line!());
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

pub fn sys_getcwd(buf: *mut u8, len: usize) -> isize {
    let process = current_process();
    info!("pid: {}", process.pid.0);
    let token = current_user_token(); 
    let mut v = translated_byte_buffer(token, buf, len);
    let inner = process.inner_exclusive_access(file!(), line!());
    let wd = inner.cwd.clone();
    let cwd = wd.get_cwd();
    if cwd.len() + 1 > len {
        return -1;
    }
    unsafe {
        let mut p = cwd.as_bytes().as_ptr();
        for slice in v.iter_mut() {
            let len = slice.len();
            ptr::copy_nonoverlapping(p, slice.as_mut_ptr(), len);
            p = p.add(len);
        }
    }
    cwd.len() as isize
}

pub fn sys_mkdir(pathname: *const u8) -> isize {
    let process = current_process();
    info!("pid: {}", process.pid.0);
    let token = current_user_token();
    let path = translated_str(token, pathname);
    let inner = process.inner_exclusive_access(file!(), line!());
    let cwd = inner.cwd.clone();
    drop(inner);
    info!("cwd inode number: {}", cwd.get_inode_id());
    let dir = cwd.create_dir(&path).clone();
    if dir.is_none() {
        return -1;
    }
    0
}

pub fn sys_fstat(fd: usize, st_addr: usize) -> isize {
    let process = current_process();
    let token = current_user_token();
    let stat = translated_refmut(token, st_addr as *mut Stat);
    let inner = process.inner_exclusive_access(file!(), line!());
    match inner.fd_table[fd].clone() {
        Some(osinode) => {
            drop(inner);
            // *stat = osinode.stat();
            let new_st = osinode.stat();
            stat.ino = new_st.ino;
            stat.mode = new_st.mode;
            stat.nlink = new_st.nlink;
            return 0;
        }
        None => {
            drop(inner);
            return -1;
        }
    }
}

pub fn sys_linkat(old_path_ptr: *const u8, new_path_ptr: *const u8) -> isize {
    let process = current_process();
    let token = current_user_token();
    let old_path = translated_str(token, old_path_ptr);
    let new_path = translated_str(token, new_path_ptr);
    let inner = process.inner_exclusive_access(file!(), line!());
    let cwd = inner.cwd.clone();
    cwd.linkat(&old_path, &new_path)
}

pub fn sys_unlinkat(path_ptr: *const u8) -> isize {
    let process = current_process();
    let token = current_user_token();
    let path = translated_str(token, path_ptr);
    let inner = process.inner_exclusive_access(file!(), line!());
    let cwd = inner.cwd.clone();
    cwd.unlinkat(&path) 
}

pub fn sys_chdir(path_ptr: *const u8) -> isize {
    let process = current_process();
    info!("pid: {}", process.pid.0);
    let token = current_user_token();
    let path = translated_str(token, path_ptr);
    let mut inner = process.inner_exclusive_access(file!(), line!());
    let cwd = inner.cwd.clone();
    if path == "/" {
        let dir = ROOT_INODE.clone();
        // inner.parent.as_mut().unwrap().upgrade().unwrap().inner_exclusive_access(file!(), line!()).cwd = dir;
        inner.cwd = dir.clone();
        return 0;
    }
    drop(inner);

    let dir = if path.clone().starts_with('/') {
        ROOT_INODE.clone().find(&path)
    } else {
        cwd.find(&path)
    };
    
    if dir.is_some() && dir.clone().unwrap().is_dir() {
        inner = process.inner_exclusive_access(file!(), line!()); 
        // inner.parent.as_mut().unwrap().upgrade().unwrap().inner_exclusive_access(file!(), line!()).cwd = dir.unwrap().clone();
        inner.cwd = dir.unwrap().clone();
    } else {
        println!("{} not exist or not a directory", path);
        return -1;
    }
    
    0
}

pub fn sys_ls() -> isize {
    let process = current_process();
    let inner = process.inner_exclusive_access(file!(), line!());
    let cwd = inner.cwd.clone();
    drop(inner);
    for app in cwd.clone().ls() {
        println!("{}", app);
    }
    0
}

// -r means remove dir recursively or normal remove for file
pub fn sys_remove(path_ptr: *const u8, args: *const u8) -> isize {
    let process = current_process();
    let token = current_user_token();
    let path = translated_str(token, path_ptr);
    let arg = translated_str(token, args);
    let inner = process.inner_exclusive_access(file!(), line!());
    let cwd = inner.cwd.clone();
    drop(inner);
    let target_inode = cwd.find(path.as_str());
    if target_inode.is_none() {
        println!("no such a file: {}", path);
        return -1;
    }
    let target_inode = target_inode.unwrap();
    let mut flag = 0;
    match target_inode.get_file_type() {
        DiskInodeType::File => {
            flag = cwd.unlinkat(&path);
            if flag == -1 {
                println!("Failed to unlink: {}", path);
            }
        },
        DiskInodeType::Directory => {
            if arg != "-r" {
                // shall be '-r'
                println!("add '-r' to remove dir");
                return -1;
            }
            remove_dir(target_inode.clone());
            flag = cwd.unlinkat(&path);
            if flag == -1 {
                println!("Failed to unlink: {}", path);
            }
        },
    };
    0
}

pub fn remove_dir(inode: Arc<Inode>) -> isize {
    match inode.get_file_type() {
        DiskInodeType::Directory => {
            let child = inode.ls();
            for c_name in child {
                if c_name == ".." || c_name == "." {
                    continue;
                }
                let c_inode = inode.find(&c_name).unwrap().clone();
                remove_dir(c_inode);
            }
            inode.clear();
        },
        DiskInodeType::File => {
            inode.clear();
        }
    };
    0
}