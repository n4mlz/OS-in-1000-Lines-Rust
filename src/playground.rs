use core::arch::asm;
use core::fmt::Write;

use crate::{
    print, println,
    process::{PM, Pid},
};

pub fn proc_a_entry() {
    loop {
        println!("A");
        PM.switch();
        for _ in 0..1000000 {
            unsafe {
                asm!("nop");
            }
        }
        println!("A");
        PM.switch();
        for _ in 0..1000000 {
            unsafe {
                asm!("nop");
            }
        }
        println!("A");
        println!("block A");
        PM.block_current();
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
        println!("B");
        PM.switch();
        for _ in 0..1000000 {
            unsafe {
                asm!("nop");
            }
        }
        println!("B");
        PM.switch();
        for _ in 0..1000000 {
            unsafe {
                asm!("nop");
            }
        }
        println!("B");
        PM.switch();
        for _ in 0..1000000 {
            unsafe {
                asm!("nop");
            }
        }
        println!("B");
        println!("unblock A");
        PM.unblock(Pid::new(1));
        PM.switch();
        for _ in 0..1000000 {
            unsafe {
                asm!("nop");
            }
        }
    }
}
