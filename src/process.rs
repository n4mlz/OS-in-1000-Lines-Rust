use core::{
    arch::{asm, naked_asm},
    cell::RefCell,
};

use crate::{
    constants::{
        FREE_RAM_END, KERNEL_BASE, KERNEL_STACK_SIZE, PAGE_R, PAGE_SIZE, PAGE_W, PAGE_X, PROCS_MAX,
        SATP_SV32,
    },
    memory::{alloc_pages, map_page},
    utils::{Addr, PhysAddr, VirtAddr},
};

#[derive(Clone, Copy, PartialEq)]
enum State {
    Unused,
    Blocked,
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
    pid: usize,
    state: State,
    page_table: PhysAddr,
    context: Context,
    sscratch: [usize; 2],
    stack: [usize; KERNEL_STACK_SIZE],
}

impl Process {
    const fn new() -> Self {
        Process {
            pid: 0,
            state: State::Unused,
            page_table: PhysAddr::NULL,
            context: Context::new(),
            sscratch: [0; 2],
            stack: [0; KERNEL_STACK_SIZE],
        }
    }
}

struct RunQueue {
    queue: RefCell<[usize; PROCS_MAX]>,
    head: RefCell<usize>,
    tail: RefCell<usize>,
}

impl RunQueue {
    const fn new() -> Self {
        RunQueue {
            queue: RefCell::new([0; PROCS_MAX]),
            head: RefCell::new(0),
            tail: RefCell::new(0),
        }
    }

    fn enqueue(&self, pid: usize) {
        let idle_pid = 0;

        if pid == idle_pid {
            return;
        }

        let tail = *self.tail.borrow();
        self.queue.borrow_mut()[tail] = pid;
        *self.tail.borrow_mut() = (tail + 1) % PROCS_MAX;
    }

    fn dequeue(&self) -> Option<usize> {
        if *self.head.borrow() == *self.tail.borrow() {
            None
        } else {
            let head = *self.head.borrow();
            let pid = self.queue.borrow()[head];
            *self.head.borrow_mut() = (head + 1) % PROCS_MAX;
            Some(pid)
        }
    }
}

pub struct ProcessManager {
    procs: [RefCell<Process>; PROCS_MAX],
    current: RefCell<usize>,
    run_queue: RunQueue,
}

impl ProcessManager {
    pub const fn new() -> Self {
        let idle_pid = 0;

        ProcessManager {
            procs: [const { RefCell::new(Process::new()) }; PROCS_MAX],
            current: RefCell::new(idle_pid),
            run_queue: RunQueue::new(),
        }
    }

    pub fn init(&self) {
        let idle_pid = 0;

        let mut idle_proc = Process::new();

        let page_table = alloc_pages(1);

        let mut paddr = unsafe { KERNEL_BASE };
        while paddr < unsafe { FREE_RAM_END } {
            map_page(
                page_table,
                VirtAddr::from_ptr(paddr),
                PhysAddr::from_ptr(paddr),
                PAGE_R | PAGE_W | PAGE_X,
            );
            paddr = unsafe { paddr.add(PAGE_SIZE) };
        }

        idle_proc.pid = 0;
        idle_proc.state = State::Runnable;
        idle_proc.page_table = page_table;
        idle_proc.sscratch = [0, idle_proc.stack.as_ptr() as usize + KERNEL_STACK_SIZE];

        self.procs[idle_pid].replace(idle_proc);
    }

    pub fn create_process(&self, pc: usize) -> Option<usize> {
        let idx = self
            .procs
            .iter()
            .position(|p| p.borrow().state == State::Unused)?;
        let mut proc = self.procs[idx].borrow_mut();

        let page_table = alloc_pages(1);

        let mut paddr = unsafe { KERNEL_BASE };
        while paddr < unsafe { FREE_RAM_END } {
            map_page(
                page_table,
                VirtAddr::from_ptr(paddr),
                PhysAddr::from_ptr(paddr),
                PAGE_R | PAGE_W | PAGE_X,
            );
            paddr = unsafe { paddr.add(PAGE_SIZE) };
        }

        proc.pid = idx;
        proc.state = State::Runnable;
        proc.page_table = page_table;
        proc.context.ra = pc;
        proc.context.sp = proc.stack.as_ptr() as usize + KERNEL_STACK_SIZE;
        proc.sscratch = [0, proc.stack.as_ptr() as usize + KERNEL_STACK_SIZE];

        self.run_queue.enqueue(proc.pid);

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
        let next = self.scheduler();
        let mut current = self.current.borrow_mut();

        if next == *current {
            return;
        }

        let mut current_proc = self.procs[*current].borrow_mut();
        let next_proc = self.procs[next].borrow();

        if current_proc.state == State::Runnable {
            self.run_queue.enqueue(*current);
        }

        *current = next;

        let current_context = &mut current_proc.context as *mut Context;
        let next_context = &next_proc.context as *const Context;

        let next_sscratch = &next_proc.sscratch;

        unsafe {
            asm!("
            sfence.vma
            csrw satp, {satp}
            sfence.vma
            csrw sscratch, {sscratch}
            ",
            satp = in(reg) SATP_SV32 | (next_proc.page_table.as_usize() / PAGE_SIZE),
            sscratch = in(reg) next_sscratch,
            );
        }

        drop(current);
        drop(current_proc);
        drop(next_proc);

        unsafe {
            Self::switch_context(current_context, next_context);
        }
    }

    pub fn block_current(&self) {
        let current = *self.current.borrow();
        let mut proc = self.procs[current].borrow_mut();
        if proc.state == State::Runnable {
            proc.state = State::Blocked;
        }
    }

    pub fn unblock(&self, pid: usize) {
        if pid == 0 {
            return;
        }

        let mut proc = self.procs[pid].borrow_mut();
        if proc.state == State::Blocked {
            proc.state = State::Runnable;
            self.run_queue.enqueue(pid);
        }
    }

    fn scheduler(&self) -> usize {
        let idle_pid = 0;

        let next = self.run_queue.dequeue();
        if let Some(pid) = next {
            return pid;
        }

        let current = *self.current.borrow();
        if self.procs[current].borrow().state == State::Runnable {
            return current;
        }

        idle_pid
    }
}

unsafe impl Sync for ProcessManager {}

pub static PM: ProcessManager = ProcessManager::new();
