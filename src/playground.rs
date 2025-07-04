use core::arch::asm;
use core::fmt::Write;

use crate::{print, println, process::PM};

pub fn proc_a_entry() {
    loop {
        println!("A");
        PM.switch();
        for _ in 0..1000000 {
            unsafe {
                asm!("nop");
            }
        }
    }
}

pub fn proc_b_entry() {
    loop {
        println!("B");
        PM.switch();
        for _ in 0..1000000 {
            unsafe {
                asm!("nop");
            }
        }
    }
}
