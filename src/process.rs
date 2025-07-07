use core::{
    arch::{asm, naked_asm},
    cell::RefCell,
};

use crate::{
    constants::{KERNEL_STACK_SIZE, PROCS_MAX},
    utils::{Addr, PhysAddr},
};

#[derive(Clone, Copy, PartialEq)]
enum State {
    Unused,
    Runnable,
}

#[derive(Clone, Copy)]
#[repr(C)]
struct Context {
    ra: usize,
    sp: usize,
    s0: usize,
    s1: usize,
    s2: usize,
    s3: usize,
    s4: usize,
    s5: usize,
    s6: usize,
    s7: usize,
    s8: usize,
    s9: usize,
    s10: usize,
    s11: usize,
}

impl Context {
    const fn new() -> Self {
        Context {
            ra: 0,
            sp: 0,
            s0: 0,
            s1: 0,
            s2: 0,
            s3: 0,
            s4: 0,
            s5: 0,
            s6: 0,
            s7: 0,
            s8: 0,
            s9: 0,
            s10: 0,
            s11: 0,
        }
    }
}

#[derive(Clone, Copy)]
struct Process {
    pid: u32,
    state: State,
    page_table: PhysAddr,
    context: Context,
    stack: [u8; KERNEL_STACK_SIZE],
}

impl Process {
    const fn new() -> Self {
        Process {
            pid: 0,
            state: State::Unused,
            page_table: PhysAddr::NULL,
            context: Context::new(),
            stack: [0; KERNEL_STACK_SIZE],
        }
    }
}

pub struct ProcessManager {
    procs: [RefCell<Process>; PROCS_MAX],
    current: RefCell<usize>,
}

impl ProcessManager {
    pub const fn new() -> Self {
        let idle_idx = 0;

        let mut pm = ProcessManager {
            procs: [const { RefCell::new(Process::new()) }; PROCS_MAX],
            current: RefCell::new(idle_idx),
        };

        let mut idle_proc = Process::new();
        idle_proc.pid = 0;
        idle_proc.state = State::Runnable;

        pm.procs[idle_idx] = RefCell::new(idle_proc);

        pm
    }

    pub fn crate_process(&self, pc: usize) -> Option<u32> {
        let idx = self
            .procs
            .iter()
            .position(|p| p.borrow().state == State::Unused)?;
        let mut proc = self.procs[idx].borrow_mut();

        proc.pid = idx as u32 + 1;
        proc.state = State::Runnable;
        proc.context.ra = pc;
        proc.context.sp = proc.stack.as_ptr() as usize + KERNEL_STACK_SIZE;

        Some(proc.pid)
    }

    #[unsafe(naked)]
    #[repr(align(4))]
    unsafe extern "C" fn switch_context(old: *mut Context, new: *const Context) {
        naked_asm!(
            "
            sw ra,  4 * 0(a0)
            sw sp, 4 * 1(a0)
            sw s0, 4 * 2(a0)
            sw s1, 4 * 3(a0)
            sw s2, 4 * 4(a0)
            sw s3, 4 * 5(a0)
            sw s4, 4 * 6(a0)
            sw s5, 4 * 7(a0)
            sw s6, 4 * 8(a0)
            sw s7, 4 * 9(a0)
            sw s8, 4 * 10(a0)
            sw s9, 4 * 11(a0)
            sw s10, 4 * 12(a0)
            sw s11, 4 * 13(a0)

            lw ra,  4 * 0(a1)
            lw sp, 4 * 1(a1)
            lw s0, 4 * 2(a1)
            lw s1, 4 * 3(a1)
            lw s2, 4 * 4(a1)
            lw s3, 4 * 5(a1)
            lw s4, 4 * 6(a1)
            lw s5, 4 * 7(a1)
            lw s6, 4 * 8(a1)
            lw s7, 4 * 9(a1)
            lw s8, 4 * 10(a1)
            lw s9, 4 * 11(a1)
            lw s10, 4 * 12(a1)
            lw s11, 4 * 13(a1)

            ret
            "
        )
    }

    pub fn switch(&self) {
        let idle_idx = 0;

        let mut current = self.current.borrow_mut();
        let next = self
            .procs
            .iter()
            .enumerate()
            .find(|&(i, proc)| {
                i != *current && proc.borrow().pid != 0 && proc.borrow().state == State::Runnable
            })
            .map(|(i, _)| i);

        if next.is_none() && self.procs[*current].borrow().state == State::Runnable {
            return;
        }

        let next = next.unwrap_or(idle_idx);

        let mut current_proc = self.procs[*current].borrow_mut();
        let next_proc = self.procs[next].borrow();

        *current = next;

        let current_context = &mut current_proc.context as *mut Context;
        let next_context = &next_proc.context as *const Context;

        let next_stack_top = next_proc.stack.as_ptr() as usize + KERNEL_STACK_SIZE;

        unsafe {
            asm!("csrw sscratch, {sscratch}",
            sscratch = in(reg) next_stack_top,
            );
        }

        drop(current);
        drop(current_proc);
        drop(next_proc);

        unsafe {
            Self::switch_context(current_context, next_context);
        }
    }
}

unsafe impl Sync for ProcessManager {}

pub static PM: ProcessManager = ProcessManager::new();
