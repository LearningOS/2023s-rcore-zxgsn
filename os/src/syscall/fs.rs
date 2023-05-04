//! File and filesystem-related syscalls

use crate::fs::{open_file, OpenFlags, Stat, link, unlink_file};
use crate::mm::{translated_byte_buffer, translated_str, UserBuffer, tran_vir_to_phy, VirtAddr, PhysAddr};
use crate::task::{current_task, current_user_token};

pub fn sys_write(fd: usize, buf: *const u8, len: usize) -> isize {
    trace!("kernel:pid[{}] sys_write", current_task().unwrap().pid.0);
    let token = current_user_token();
    let task = current_task().unwrap();
    let inner = task.inner_exclusive_access();
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
    trace!("kernel:pid[{}] sys_read", current_task().unwrap().pid.0);
    let token = current_user_token();
    let task = current_task().unwrap();
    let inner = task.inner_exclusive_access();
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
        trace!("kernel: sys_read .. file.read");
        file.read(UserBuffer::new(translated_byte_buffer(token, buf, len))) as isize
    } else {
        -1
    }
}

pub fn sys_open(path: *const u8, flags: u32) -> isize {
    trace!("kernel:pid[{}] sys_open", current_task().unwrap().pid.0);
    let task = current_task().unwrap();
    let token = current_user_token();
    let path = translated_str(token, path);
    if let Some(inode) = open_file(path.as_str(), OpenFlags::from_bits(flags).unwrap()) {
        let mut inner = task.inner_exclusive_access();
        let fd = inner.alloc_fd();
        inner.fd_table[fd] = Some(inode);
        fd as isize
    } else {
        -1
    }
}

pub fn sys_close(fd: usize) -> isize {
    trace!("kernel:pid[{}] sys_close", current_task().unwrap().pid.0);
    let task = current_task().unwrap();
    let mut inner = task.inner_exclusive_access();
    if fd >= inner.fd_table.len() {
        return -1;
    }
    if inner.fd_table[fd].is_none() {
        return -1;
    }
    inner.fd_table[fd].take();
    0
}

/// YOUR JOB: Implement fstat.
pub fn sys_fstat(_fd: usize, _st: *mut Stat) -> isize {
    trace!("kernel:pid[{}] sys_fstat", current_task().unwrap().pid.0);
    println!("fstat begin");
    let token = current_user_token();
    let task = current_task().unwrap();
    let inner = task.inner_exclusive_access();
    if _fd >= inner.fd_table.len() {
        return -1;
    }
    if inner.fd_table[_fd].is_none() {
        return -1;
    }
    
    // println!("fstat end0");
    let vaddr: VirtAddr = (_st as usize).into();
    // println!("fstat end-1");
    let offset = vaddr.page_offset();
    // println!("fstat end1");
    let ppn = tran_vir_to_phy(token, vaddr);
    let start_addr = PhysAddr::from(ppn);
    let phy_addr = PhysAddr::from(usize::from(start_addr) + offset);
    let st = phy_addr.get_mut::<Stat>();
    // println!("fstat end2");
    if let Some(file) = &inner.fd_table[_fd] {
        println!("fstat end");
        *st = file.stat();
        0
    } else {
        -1
    }
}

/// YOUR JOB: Implement linkat.
pub fn sys_linkat(_old_name: *const u8, _new_name: *const u8) -> isize {
    trace!("kernel:pid[{}] sys_linkat", current_task().unwrap().pid.0);
    // let task = current_task().unwrap();
    // let mut inner = task.inner_exclusive_access();
    println!("link begin");
    let token = current_user_token();
    let old_name = translated_str(token, _old_name);
    let new_name = translated_str(token, _new_name);
    if old_name.eq(&new_name) {
        return -1;
    };
    link(old_name, new_name);
    println!("link end");
    0
}

/// YOUR JOB: Implement unlinkat.
pub fn sys_unlinkat(_name: *const u8) -> isize {
    println!("unlink begin");
    trace!("kernel:pid[{}] sys_unlinkat", current_task().unwrap().pid.0);
    let token = current_user_token();
    let name = translated_str(token, _name);
    if unlink_file(&name).is_none() {
        return -1;
    }
    0
}
