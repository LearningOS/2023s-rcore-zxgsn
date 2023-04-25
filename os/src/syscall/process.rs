//! Process management syscalls
// use riscv::addr::page;

// use alloc::vec::Vec;

use crate::{
    config::MAX_SYSCALL_NUM,
    mm::{successful_map, successful_unmap, tran_vir_to_phy, VirtAddr},
    task::{
        change_program_brk, current_user_token, exit_current_and_run_next,
        suspend_current_and_run_next, TaskStatus,
    },
    timer::get_time_us,
};

#[repr(C)]
#[derive(Debug)]
pub struct TimeVal {
    pub sec: usize,
    pub usec: usize,
}

/// Task information
#[allow(dead_code)]
pub struct TaskInfo {
    /// Task status in it's life cycle
    status: TaskStatus,
    /// The numbers of syscall called by task
    syscall_times: [u32; MAX_SYSCALL_NUM],
    /// Total running time of task
    time: usize,
}

/// task exits and submit an exit code
pub fn sys_exit(_exit_code: i32) -> ! {
    trace!("kernel: sys_exit");
    exit_current_and_run_next();
    panic!("Unreachable in sys_exit!");
}

/// current task gives up resources for other tasks
pub fn sys_yield() -> isize {
    trace!("kernel: sys_yield");
    suspend_current_and_run_next();
    0
}

/// YOUR JOB: get time with second and microsecond
/// HINT: You might reimplement it with virtual memory management.
/// HINT: What if [`TimeVal`] is splitted by two pages ?
pub fn sys_get_time(_ts: *mut TimeVal, _tz: usize) -> isize {
    trace!("kernel: sys_get_time");
    // token存的是当前多级页表的根节点所在的物理页号
    let token = current_user_token();
    // let usr_token = current_user_token();
    // 用户空间的虚拟地址
    let vaddr: VirtAddr = (_ts as usize).into();
    let ppn = tran_vir_to_phy(token, vaddr);
    let us = get_time_us();
    let vaddr2: VirtAddr = ((_ts as usize) + 8).into();
    let ppn2 = tran_vir_to_phy(token, vaddr2);
    // println!("1 = {}, 2 = {}", usize::from(ppn), usize::from(ppn2));
    if ppn != ppn2 {
        println!("不在同一个页面上");
        let tv = ppn.get_mut::<usize>();
        let tv2 = ppn2.get_mut::<usize>();
        *tv = us / 1_000_000;
        *tv2 = us % 1_000_000;
    } else {
        let tv = ppn.get_mut::<TimeVal>();
        // println!("写入页面：{}", ppn.0);
        *tv = TimeVal {
            sec: us / 1_000_000,
            usec: us % 1_000_000,
        };
    }

    0
}

/// YOUR JOB: Finish sys_task_info to pass testcases
/// HINT: You might reimplement it with virtual memory management.
/// HINT: What if [`TaskInfo`] is splitted by two pages ?
pub fn sys_task_info(_ti: *mut TaskInfo) -> isize {
    trace!("kernel: sys_task_info NOT IMPLEMENTED YET!");
    -1
}

// YOUR JOB: Implement mmap.
pub fn sys_mmap(_start: usize, _len: usize, _port: usize) -> isize {
    trace!("kernel: sys_mmap");
    // 未按页对齐
    if _start % 4096 != 0 {
        return -1;
    }
    if (_port & !0x7 != 0) || (_port & 0x7 == 0) {
        return -1;
    }
    //let token = current_user_token();
    if successful_map(_start, _len, _port) {
        0
    } else {
        -1
    }
}

// YOUR JOB: Implement munmap.
pub fn sys_munmap(_start: usize, _len: usize) -> isize {
    trace!("kernel: sys_munmap");
    let token = current_user_token();
    if successful_unmap(token, _start, _len) {
        0
    } else {
        -1
    }
}
/// change data segment size
pub fn sys_sbrk(size: i32) -> isize {
    trace!("kernel: sys_sbrk");
    if let Some(old_brk) = change_program_brk(size) {
        old_brk as isize
    } else {
        -1
    }
}
