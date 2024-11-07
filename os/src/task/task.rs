//! Types related to task management

use super::TaskContext;
use crate::config::{MAX_SYSCALL_NUM, TRAP_CONTEXT_BASE};
use crate::mm::{
    kernel_stack_position, MapPermission, MemorySet, PhysPageNum, VirtAddr, KERNEL_SPACE,
};
use crate::timer::get_time_us;
use crate::trap::{trap_handler, TrapContext};

/// The task control block (TCB) of a task.
pub struct TaskControlBlock {
    /// Save task context
    pub task_cx: TaskContext,

    /// Maintain the execution status of the current process
    pub task_status: TaskStatus,

    /// Application address space
    pub memory_set: MemorySet,

    /// The phys page number of trap context
    pub trap_cx_ppn: PhysPageNum,

    /// The size(top addr) of program which is loaded from elf file
    pub base_size: usize,

    /// Heap bottom
    pub heap_bottom: usize,

    /// Program break
    pub program_brk: usize,

    /// The numbers of syscall called by task
    pub syscall_times: [u32; MAX_SYSCALL_NUM],
    /// Total running time of task
    pub time: usize,
}

impl TaskControlBlock {
    /// get the trap context
    pub fn get_trap_cx(&self) -> &'static mut TrapContext {
        self.trap_cx_ppn.get_mut()
    }
    /// get the user token
    pub fn get_user_token(&self) -> usize {
        self.memory_set.token()
    }
    /// Based on the elf info in program, build the contents of task in a new address space
    pub fn new(elf_data: &[u8], app_id: usize) -> Self {
        // memory_set with elf program headers/trampoline/trap context/user stack
        let (memory_set, user_sp, entry_point) = MemorySet::from_elf(elf_data);
        let trap_cx_ppn = memory_set
            .translate(VirtAddr::from(TRAP_CONTEXT_BASE).into())
            .unwrap()
            .ppn();
        let task_status = TaskStatus::Ready;
        // map a kernel-stack in kernel space
        let (kernel_stack_bottom, kernel_stack_top) = kernel_stack_position(app_id);
        KERNEL_SPACE.exclusive_access().insert_framed_area(
            kernel_stack_bottom.into(),
            kernel_stack_top.into(),
            MapPermission::R | MapPermission::W,
        );
        let task_control_block = Self {
            task_status,
            task_cx: TaskContext::goto_trap_return(kernel_stack_top),
            memory_set,
            trap_cx_ppn,
            base_size: user_sp,
            heap_bottom: user_sp,
            program_brk: user_sp,
            syscall_times: [0; MAX_SYSCALL_NUM],
            time: get_time_us(),
        };
        // prepare TrapContext in user space
        let trap_cx = task_control_block.get_trap_cx();
        *trap_cx = TrapContext::app_init_context(
            entry_point,
            user_sp,
            KERNEL_SPACE.exclusive_access().token(),
            kernel_stack_top,
            trap_handler as usize,
        );
        task_control_block
    }
    /// change the location of the program break. return None if failed.
    pub fn change_program_brk(&mut self, size: i32) -> Option<usize> {
        let old_break = self.program_brk;
        let new_brk = self.program_brk as isize + size as isize;
        if new_brk < self.heap_bottom as isize {
            return None;
        }
        let result = if size < 0 {
            self.memory_set
                .shrink_to(VirtAddr(self.heap_bottom), VirtAddr(new_brk as usize))
        } else {
            self.memory_set
                .append_to(VirtAddr(self.heap_bottom), VirtAddr(new_brk as usize))
        };
        if result {
            self.program_brk = new_brk as usize;
            Some(old_break)
        } else {
            None
        }
    }
    pub fn program_mmap(&mut self, _start: usize, _len: usize, _port: usize) -> bool {
        // let old_break = self.program_brk;
        // let new_brk = self.program_brk as isize + size as isize;
        if _len == 0 ||
        _port & !0x7 !=0 ||
        _port & 0x7 == 0
        {
            return false;
        }


        let _end = _start + _len;
        // if let Some(perm) = MapPermission::from_bits((_port << 1 | 1 << 4) as u8){
        //     self.memory_set
        //     .insert_framed_area(VirtAddr(_start), VirtAddr(_end), perm)
        // } else {
        //     false
        // }
        let  mut perm =MapPermission::empty();
        perm.set(MapPermission::R, _port & 0x1 != 0);
        perm.set(MapPermission::W, _port & 0x2 != 0);
        perm.set(MapPermission::X, _port & 0x4 != 0);
        perm.set(MapPermission::U, true);
        self.memory_set
            .insert_framed_area(VirtAddr(_start), VirtAddr(_end), perm)
    }
    pub fn program_mummap(&mut self, _start: usize, _len: usize) -> bool {
        // let old_break = self.program_brk;
        // let new_brk = self.program_brk as isize + size as isize;
        if _len == 0 {
            return false;
        }
        let _end = _start + _len;
        
        self.memory_set
                .delete_framed_area(VirtAddr(_start), VirtAddr(_end))
    }

    pub fn syscall_count(&mut self, syscall_id: usize) {
        self.syscall_times[syscall_id] += 1;
    }
}

#[derive(Copy, Clone, PartialEq)]
/// task status: UnInit, Ready, Running, Exited
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
