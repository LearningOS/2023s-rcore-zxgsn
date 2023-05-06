use crate::sync::{Condvar, Mutex, MutexBlocking, MutexSpin, Semaphore};
use crate::task::{block_current_and_run_next, current_process, current_task};
use crate::timer::{add_timer, get_time_ms};
use alloc::collections::BTreeSet;
use alloc::sync::Arc;
use alloc::vec::Vec;
/// sleep syscall
pub fn sys_sleep(ms: usize) -> isize {
    trace!(
        "kernel:pid[{}] tid[{}] sys_sleep",
        current_task().unwrap().process.upgrade().unwrap().getpid(),
        current_task()
            .unwrap()
            .inner_exclusive_access()
            .res
            .as_ref()
            .unwrap()
            .tid
    );
    let expire_ms = get_time_ms() + ms;
    let task = current_task().unwrap();
    add_timer(expire_ms, task);
    block_current_and_run_next();
    0
}
/// mutex create syscall
pub fn sys_mutex_create(blocking: bool) -> isize {
    trace!(
        "kernel:pid[{}] tid[{}] sys_mutex_create",
        current_task().unwrap().process.upgrade().unwrap().getpid(),
        current_task()
            .unwrap()
            .inner_exclusive_access()
            .res
            .as_ref()
            .unwrap()
            .tid
    );
    let process = current_process();
    let mutex: Option<Arc<dyn Mutex>> = if !blocking {
        Some(Arc::new(MutexSpin::new()))
    } else {
        Some(Arc::new(MutexBlocking::new()))
    };

    let mut process_inner = process.inner_exclusive_access();
    // process_inner.mutex_list[0] = mutex.clone();
    if let Some(id) = process_inner
        .mutex_list
        .iter()
        .enumerate()
        .find(|(_, item)| item.is_none())
        .map(|(id, _)| id)
    {
        process_inner.mutex_list[id] = mutex;
        id as isize
    } else {
        process_inner.mutex_list.push(mutex);
        process_inner.mutex_list.len() as isize - 1
    }
}
/// mutex lock syscall
pub fn sys_mutex_lock(mutex_id: usize) -> isize {
    trace!(
        "kernel:pid[{}] tid[{}] sys_mutex_lock",
        current_task().unwrap().process.upgrade().unwrap().getpid(),
        current_task()
            .unwrap()
            .inner_exclusive_access()
            .res
            .as_ref()
            .unwrap()
            .tid
    );
    let tid = current_task()
        .unwrap()
        .inner_exclusive_access()
        .res
        .as_ref()
        .unwrap()
        .tid;
    let process = current_process();
    let mut process_inner = process.inner_exclusive_access();
    let mutex = Arc::clone(process_inner.mutex_list[mutex_id].as_ref().unwrap());
    // 表示该进程tid需要mid的锁
    // println!("tid = {}", tid);
    // 要改一下长度， 不然vec未被初始化不能访问
    process_inner.mutex_request.resize(17, None);
    process_inner.mutex_alloc.resize(17, None);
    process_inner.mutex_request[tid] = Some(mutex_id);
    let detection = process_inner.deadlock_detection;
    // 判断能不能分配
    // println!("here 2");
    if detection {
        let mut visited = BTreeSet::<usize>::new();
        let mut mid = mutex_id;
        visited.insert(tid);
        // 下面获取的是正在占有锁的线程
        // println!("here 3");
        while let Some(new_tid) = process_inner.mutex_alloc[mid] {
            // println!("here 4");
            if visited.contains(&new_tid) {
                // 说明此线程在等待其他线程，且无法解除等待，即发生死锁
                // println!("deadlock!");
                return -0xdead;
            } else {
                visited.insert(new_tid);
                // 检查tid的线程是否发生死锁
                let new_mid = process_inner.mutex_request[tid];
                if new_mid.is_some() {
                    // 为下一轮循环做准备
                    mid = new_mid.unwrap();
                } else {
                    // 安全
                    break;
                }
            }
        }
    }
    drop(process_inner);
    drop(process);
    mutex.lock();
    let process = current_process();
    let mut process_inner = process.inner_exclusive_access();
    process_inner.mutex_request[tid] = None;
    process_inner.mutex_alloc[mutex_id] = Some(tid);
    0
}
/// mutex unlock syscall
pub fn sys_mutex_unlock(mutex_id: usize) -> isize {
    trace!(
        "kernel:pid[{}] tid[{}] sys_mutex_unlock",
        current_task().unwrap().process.upgrade().unwrap().getpid(),
        current_task()
            .unwrap()
            .inner_exclusive_access()
            .res
            .as_ref()
            .unwrap()
            .tid
    );
    let process = current_process();
    let process_inner = process.inner_exclusive_access();
    let mutex = Arc::clone(process_inner.mutex_list[mutex_id].as_ref().unwrap());
    drop(process_inner);
    drop(process);
    mutex.unlock();
    0
}
/// semaphore create syscall
pub fn sys_semaphore_create(res_count: usize) -> isize {
    trace!(
        "kernel:pid[{}] tid[{}] sys_semaphore_create",
        current_task().unwrap().process.upgrade().unwrap().getpid(),
        current_task()
            .unwrap()
            .inner_exclusive_access()
            .res
            .as_ref()
            .unwrap()
            .tid
    );
    let process = current_process();
    let mut process_inner = process.inner_exclusive_access();
    let id = if let Some(id) = process_inner
        .semaphore_list
        .iter()
        .enumerate()
        .find(|(_, item)| item.is_none())
        .map(|(id, _)| id)
    {
        process_inner.semaphore_list[id] = Some(Arc::new(Semaphore::new(res_count)));
        id
    } else {
        process_inner
            .semaphore_list
            .push(Some(Arc::new(Semaphore::new(res_count))));
        process_inner.semaphore_list.len() - 1
    };

    if let Some(id) = process_inner
        .semaphore_list
        .iter()
        .enumerate()
        .find(|(_, item)| item.is_none())
        .map(|(id, _)| id)
    {
        process_inner.semaphore_available[id] += res_count;
        
    } else {
        process_inner
            .semaphore_available
            .push(res_count);
        
    };
    // println!("ava_size1 {}", process_inner.semaphore_available.len());
    
    id as isize
}
/// semaphore up syscall
pub fn sys_semaphore_up(sem_id: usize) -> isize {
    trace!(
        "kernel:pid[{}] tid[{}] sys_semaphore_up",
        current_task().unwrap().process.upgrade().unwrap().getpid(),
        current_task()
            .unwrap()
            .inner_exclusive_access()
            .res
            .as_ref()
            .unwrap()
            .tid
    );
    let tid = current_task()
        .unwrap()
        .inner_exclusive_access()
        .res
        .as_ref()
        .unwrap()
        .tid;
    let process = current_process();
    let mut process_inner = process.inner_exclusive_access();
    let sem = Arc::clone(process_inner.semaphore_list[sem_id].as_ref().unwrap());
    process_inner.semaphore_alloc[tid][sem_id] -= 1;
    process_inner.semaphore_available[sem_id] += 1;
    drop(process_inner);
    sem.up();
    0
}
/// semaphore down syscall
pub fn sys_semaphore_down(sem_id: usize) -> isize {
    trace!(
        "kernel:pid[{}] tid[{}] sys_semaphore_down",
        current_task().unwrap().process.upgrade().unwrap().getpid(),
        current_task()
            .unwrap()
            .inner_exclusive_access()
            .res
            .as_ref()
            .unwrap()
            .tid
    );
    let tid = current_task()
        .unwrap()
        .inner_exclusive_access()
        .res
        .as_ref()
        .unwrap()
        .tid;
    // println!("here1");
    let process = current_process();
    let mut process_inner = process.inner_exclusive_access();
    let sem = Arc::clone(process_inner.semaphore_list[sem_id].as_ref().unwrap());
    // 此时的死锁检测多了一个维度
    let sid = sem_id;
    
    process_inner.semaphore_request[tid][sid] = 1;
    // println!("process {} need sem {}", tid, sid);
    // step1
    let mut work = process_inner.semaphore_available.clone();
    // println!("ava_size {}", process_inner.semaphore_available.len());
    // println!("work_size {}", work.len());
    // work.resize(10, 0);
    // 保存未完成的线程
    let mut unfinish = BTreeSet::<usize>::new();
    // unfinish.insert(tid);
    let count = process_inner.thread_count();
    let mut finish = Vec::new();
    // 用不了vec宏只能用循环初始化了
    for _ in 0..count {
        finish.push(false);
    }
    /*
    let mut finish: Vec<bool> = Vec::with_capacity(count);
    finish.fill(false);
    实际上，Vec::with_capacity() 方法只是为 finish 分配了内存空间，
    并没有对数组中的元素进行初始化。因此，finish 数组的实际长度仍然为 0。
    */
    // println!("here2");
    // step2
    while let Some((new_tid, _)) = finish.iter().enumerate().find(|(_, val)| **val == false) {
        // println!("here3");
        if unfinish.contains(&new_tid) {
            // println!("process: {} need: sem id{}", new_tid, sid);
            return -0xdead;
        }
        unfinish.insert(new_tid);
        // println!("insert {}", new_tid);
        // println!("hreeee");
        // println!("request: {}, work{}", process_inner.semaphore_request[new_tid][sid], work[sid]);
        if process_inner.semaphore_request[new_tid][sid] <= work[sid] {
            // step3
            // println!("here0");
            work[sid] += process_inner.semaphore_alloc[new_tid][sid];
            // println!("here10");
            finish[new_tid] = true;
            unfinish.remove(&new_tid);
            // println!("remove {}", new_tid);
        }
        // println!("here4");
    }
    // println!("here5");
    // println!("over");
    drop(process_inner);
    sem.down();
    // 重置一下向量
    let process = current_process();
    let mut process_inner = process.inner_exclusive_access();
    // let sem = Arc::clone(process_inner.semaphore_list[sem_id].as_ref().unwrap());
    process_inner.semaphore_alloc[tid][sem_id] += 1;
    process_inner.semaphore_available[sem_id] -= 1;
    process_inner.semaphore_request[tid][sem_id] = 0;
    0
}
/// condvar create syscall
pub fn sys_condvar_create() -> isize {
    trace!(
        "kernel:pid[{}] tid[{}] sys_condvar_create",
        current_task().unwrap().process.upgrade().unwrap().getpid(),
        current_task()
            .unwrap()
            .inner_exclusive_access()
            .res
            .as_ref()
            .unwrap()
            .tid
    );
    let process = current_process();
    let mut process_inner = process.inner_exclusive_access();
    let id = if let Some(id) = process_inner
        .condvar_list
        .iter()
        .enumerate()
        .find(|(_, item)| item.is_none())
        .map(|(id, _)| id)
    {
        process_inner.condvar_list[id] = Some(Arc::new(Condvar::new()));
        id
    } else {
        process_inner
            .condvar_list
            .push(Some(Arc::new(Condvar::new())));
        process_inner.condvar_list.len() - 1
    };
    id as isize
}
/// condvar signal syscall
pub fn sys_condvar_signal(condvar_id: usize) -> isize {
    trace!(
        "kernel:pid[{}] tid[{}] sys_condvar_signal",
        current_task().unwrap().process.upgrade().unwrap().getpid(),
        current_task()
            .unwrap()
            .inner_exclusive_access()
            .res
            .as_ref()
            .unwrap()
            .tid
    );
    let process = current_process();
    let process_inner = process.inner_exclusive_access();
    let condvar = Arc::clone(process_inner.condvar_list[condvar_id].as_ref().unwrap());
    drop(process_inner);
    condvar.signal();
    0
}
/// condvar wait syscall
pub fn sys_condvar_wait(condvar_id: usize, mutex_id: usize) -> isize {
    trace!(
        "kernel:pid[{}] tid[{}] sys_condvar_wait",
        current_task().unwrap().process.upgrade().unwrap().getpid(),
        current_task()
            .unwrap()
            .inner_exclusive_access()
            .res
            .as_ref()
            .unwrap()
            .tid
    );
    let process = current_process();
    let process_inner = process.inner_exclusive_access();
    let condvar = Arc::clone(process_inner.condvar_list[condvar_id].as_ref().unwrap());
    let mutex = Arc::clone(process_inner.mutex_list[mutex_id].as_ref().unwrap());
    drop(process_inner);
    condvar.wait(mutex);
    0
}
/// enable deadlock detection syscall
///
/// YOUR JOB: Implement deadlock detection, but might not all in this syscall
pub fn sys_enable_deadlock_detect(_enabled: usize) -> isize {
    trace!("kernel: sys_enable_deadlock_detect");
    let process = current_process();
    let mut inner = process.inner_exclusive_access();
    match _enabled {
        0 => {
            inner.deadlock_detection = false;
            0
        }
        1 => {
            inner.deadlock_detection = true;
            0
        }
        _ => -1,
    }
}
