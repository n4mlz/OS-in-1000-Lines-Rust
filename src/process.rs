use crate::constants::KERNEL_STACK_SIZE;

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
