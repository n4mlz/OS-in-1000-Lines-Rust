#![no_main]
#![no_std]
#![feature(fn_align)]

mod constants;
mod sbi;
mod trap_handler;
mod utils;

use crate::{
    constants::{BSS, BSS_END, STACK_TOP},
    trap_handler::kernel_entry,
};
use core::{arch::asm, fmt::Write, panic::PanicInfo, ptr};

#[unsafe(no_mangle)]
#[unsafe(link_section = ".text.boot")]
extern "C" fn boot() -> ! {
    unsafe {
        asm!(
            "mv sp, {stack_top}
            j {kernel_main}",
            stack_top = in(reg) STACK_TOP,
            kernel_main = sym kernel_main,
            options(noreturn)
        );
    }
}

fn kernel_main() -> ! {
    unsafe {
        ptr::write_bytes(BSS, 0, BSS_END.offset_from(BSS) as usize);

        write_csr!("stvec", kernel_entry);
    }

    println!("Hello, World!");

    unsafe { asm!("unimp") };

    unreachable!();
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{info}");
    loop {
        unsafe {
            asm!("wfi");
        }
    }
}
