//!Implementation of [`TaskManager`]
use super::TaskControlBlock;
use crate::sync::UPSafeCell;
use alloc::collections::VecDeque;
use alloc::sync::Arc;
use lazy_static::*;
///A array of `TaskControlBlock` that is thread-safe
pub struct TaskManager {
    big_stride: isize,
    ready_queue: VecDeque<Arc<TaskControlBlock>>,
}

/// A simple FIFO scheduler.
impl TaskManager {
    ///Creat an empty TaskManager
    pub fn new() -> Self {
        Self {
            big_stride: 255,
            ready_queue: VecDeque::new(),
        }
    }
    /// Add process back to ready queue
    pub fn add(&mut self, task: Arc<TaskControlBlock>) {
        self.ready_queue.push_back(task);
    }
    /// Take a process out of the ready queue
    pub fn fetch(&mut self) -> Option<Arc<TaskControlBlock>> {
        // self.ready_queue.pop_front()
        // println!("size of vec {}", self.ready_queue.len());
        let mut min = 10000;
        let mut index:usize = 0;
        /*for (index, tcb) in self.ready_queue.iter().enumerate() {
            let stride = tcb.get_stride();
            println!("Task {} has stride {}", index, stride);
        }*/
        for i in 0..self.ready_queue.len() {
            if min > self.ready_queue[i].get_stride() {
                min = self.ready_queue[i].get_stride();
                index = i;
            }
        }
        let priority = self.ready_queue[index].get_priority();
        self.ready_queue[index].inner_exclusive_access().stride +=  self.big_stride / priority;
        // set_stride(self.big_stride / priority);
        // println!("index {}, set stride: {}", index, self.big_stride / priority);
        /*let (index, _) = self
            .ready_queue
            .iter()
            .enumerate()
            .min_by_key(|(_, tcb)| tcb.get_stride())
            .unwrap();*/
        
        // println!("set stride {}", self.ready_queue[index].get_stride());
        self.ready_queue.remove(index)
    }
}

lazy_static! {
    /// TASK_MANAGER instance through lazy_static!
    pub static ref TASK_MANAGER: UPSafeCell<TaskManager> =
        unsafe { UPSafeCell::new(TaskManager::new()) };
}

/// Add process to ready queue
pub fn add_task(task: Arc<TaskControlBlock>) {
    //trace!("kernel: TaskManager::add_task");
    TASK_MANAGER.exclusive_access().add(task);
}

/// Take a process out of the ready queue
pub fn fetch_task() -> Option<Arc<TaskControlBlock>> {
    //trace!("kernel: TaskManager::fetch_task");
    TASK_MANAGER.exclusive_access().fetch()
}
