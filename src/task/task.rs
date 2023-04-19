//! Types related to task management

use super::TaskContext;
use crate::config::MAX_SYSCALL_NUM;

/// The task control block (TCB) of a task.
#[derive(Copy, Clone)]
pub struct TaskControlBlock {
    /// The task status in it's lifecycle
    pub task_status: TaskStatus,
    /// The task context
    pub task_cx: TaskContext,
    /// the beginning time
    pub time : usize,
    /// syscall_times  why this is a bad idea
    pub tcb_syscall_times : [u32; MAX_SYSCALL_NUM],
}
/*
impl TaskControlBlock {
    pub fn get_tcb_syscall_times(&self) -> [u32; MAX_SYSCALL_NUM] {
        self.tcb_syscall_times
    }

    pub fn change_tcb_syscall_times(&self) {

    }
}
*/

/// The status of a task
#[derive(Copy, Clone, PartialEq)]
pub enum TaskStatus {
    /// uninitialized
    UnInit,
    /// ready to run
    Ready,
    /// running
    Running,
    /// exited
    Exited,
}
