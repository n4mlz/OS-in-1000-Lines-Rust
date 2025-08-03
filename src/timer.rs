use core::arch::asm;

use crate::{constants::TIMER_QUANTUM_US, sbi::sbi_call, utils::irq_enable, write_csr_set};

const SBI_EID_TIME: usize = 0x54494d45;
const SBI_FID_SET_TIMER: usize = 0;

const TIMEBASE_FREQ: u64 = 10_000_000; // 10 MHz

fn get_time() -> u64 {
    let (mut hi, mut lo, mut tmp): (u32, u32, u32);
    loop {
        unsafe {
            asm!("rdtimeh {0}", out(reg) hi, options(nomem, nostack));
            asm!("rdtime  {0}", out(reg) lo, options(nomem, nostack));
            asm!("rdtimeh {0}", out(reg) tmp, options(nomem, nostack));
        }
        if hi == tmp {
            break;
        }
    }
    ((hi as u64) << 32) | lo as u64
}

fn set_next_timer() {
    let now = get_time();
    let delta = TIMER_QUANTUM_US * TIMEBASE_FREQ / 1_000_000;
    let next = now + delta;

    let lo = next as u32 as usize;
    let hi = (next >> 32) as u32 as usize;

    let sbi_ret = sbi_call(lo, hi, 0, 0, 0, 0, SBI_EID_TIME, SBI_FID_SET_TIMER);
    if let Err(err) = sbi_ret {
        panic!("Failed to set timer: {err}");
    }
}

pub fn enable_timer_irq() {
    unsafe {
        write_csr_set!("sie", 1 << 5); // STIE
        irq_enable();
    }
}

pub fn init_timer() {
    enable_timer_irq();
    set_next_timer();
}

pub fn handle_timer_irq() {
    set_next_timer();

    let now = get_time();
}
