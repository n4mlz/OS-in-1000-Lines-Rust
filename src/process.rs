use core::arch::naked_asm;

use crate::{
    constants::KERNEL_STACK_SIZE,
    utils::{Addr, VirtAddr},
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
    fn new() -> Self {
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
    context: Context,
    stack: [u8; KERNEL_STACK_SIZE],
}

impl Process {
    fn new() -> Self {
        Process {
            pid: 0,
            state: State::Unused,
            context: Context::new(),
            stack: [0; KERNEL_STACK_SIZE],
        }
    }
}

struct ProcessManager {
    procs: [Process; 8],
    current: usize,
}

impl ProcessManager {
    fn new() -> Self {
        let idle_idx = 0;

        let mut pm = ProcessManager {
            procs: [Process::new(); 8],
            current: idle_idx,
        };

        let proc = &mut pm.procs[idle_idx];
        proc.pid = 0;
        proc.state = State::Runnable;

        pm
    }

    fn crate_process(&mut self, pc: VirtAddr) -> Option<u32> {
        let idx = self.procs.iter().position(|p| p.state == State::Unused)?;
        let proc = &mut self.procs[idx];

        proc.pid = idx as u32 + 1;
        proc.state = State::Runnable;
        proc.context.ra = pc.as_usize();
        proc.context.sp = proc.stack.as_ptr() as usize + KERNEL_STACK_SIZE;

        Some(proc.pid)
    }

    #[unsafe(naked)]
    #[repr(align(4))]
    unsafe extern "C" fn switch_context(old: &mut Context, new: &Context) {
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

    fn switch(&mut self) {
        let current = self.current;
        let next = self
            .procs
            .iter()
            .enumerate()
            .find(|&(i, proc)| i != current && proc.pid != 0 && proc.state == State::Runnable)
            .map(|(i, _)| i);

        if next.is_none() && self.procs[current].state == State::Runnable {
            return;
        }

        let next = next.unwrap_or(0);

        let (current_proc, next_proc) = if current < next {
            let (left, right) = self.procs.split_at_mut(next);
            (&mut left[current], &right[0])
        } else {
            let (left, right) = self.procs.split_at_mut(current);
            (&mut left[0], &right[next])
        };

        unsafe {
            Self::switch_context(&mut current_proc.context, &next_proc.context);
        }

        self.current = next;
    }
}
