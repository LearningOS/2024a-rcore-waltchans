//! Process management syscalls

use crate::{
    config::MAX_SYSCALL_NUM,
    mm::translated_byte_buffer,
    task::{
        change_program_brk, current_user_token, exit_current_and_run_next, get_task_first_run_time, get_task_syscall_times, program_mmap, program_mummap, suspend_current_and_run_next, TaskStatus
    }, 
    timer::{
        get_time_ms, get_time_us
    },
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

fn write_into_buffer(token: usize, ptr: *const u8, len: usize, src: *const u8) -> bool {
    let mut dst_vec = translated_byte_buffer(token, ptr, len);
    // let insize = 0;
    unsafe {
        dst_vec[0].copy_from_slice(core::slice::from_raw_parts(src, len));
    }
    // for dst in dst_vec {
    //     let mut blen = dst.len();
    //     if blen > len - insize {
    //         blen = len - insize;
    //     }
    //     unsafe {
    //         let bufsrc = src.add(insize);
    //         dst.copy_from_slice(core::slice::from_raw_parts(bufsrc, blen));
    //     };
    // }
    false
}

/// YOUR JOB: get time with second and microsecond
/// HINT: You might reimplement it with virtual memory management.
/// HINT: What if [`TimeVal`] is splitted by two pages ?
pub fn sys_get_time(_ts: *mut TimeVal, _tz: usize) -> isize {
    trace!("kernel: sys_get_time");
    let us = get_time_us();
    let time_info=TimeVal {
        sec: us / 1_000_000,
        usec: us % 1_000_000,
    };
    // let buffers = translated_byte_buffer(current_user_token(), _ts as *mut u8, _tz);
    
    write_into_buffer(current_user_token(), _ts as *const u8, _tz, &time_info as *const TimeVal as *const u8);
    0
}

/// YOUR JOB: Finish sys_task_info to pass testcases
/// HINT: You might reimplement it with virtual memory management.
/// HINT: What if [`TaskInfo`] is splitted by two pages ?
pub fn sys_task_info(_ti: *mut TaskInfo) -> isize {
    trace!("kernel: sys_task_info NOT IMPLEMENTED YET!");
    let taskinfo= TaskInfo {
            status: TaskStatus::Running,
            syscall_times: get_task_syscall_times(),
            time: get_time_ms() - get_task_first_run_time(),
        };
    let size = core::mem::size_of::<TaskInfo>();

    write_into_buffer(current_user_token(), _ti as *const u8, size, &taskinfo as *const TaskInfo as *const u8);
    0
}


// YOUR JOB: Implement mmap.
pub fn sys_mmap(_start: usize, _len: usize, _port: usize) -> isize {
    trace!("kernel: sys_mmap NOT IMPLEMENTED YET!");
    if program_mmap(_start, _len, _port) {
        0
    } else {
        -1
    }
}

// YOUR JOB: Implement munmap.
pub fn sys_munmap(_start: usize, _len: usize) -> isize {
    trace!("kernel: sys_munmap NOT IMPLEMENTED YET!");
    if program_mummap(_start, _len) {
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
