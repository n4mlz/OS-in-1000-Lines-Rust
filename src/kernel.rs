#![no_main]
#![no_std]
#![feature(fn_align)]

mod constants;
mod memory;
mod sbi;
mod trap_handler;
mod utils;

use core::{arch::asm, fmt::Write, panic::PanicInfo, ptr};

use crate::{
    constants::{BSS, BSS_END, STACK_TOP},
    memory::alloc_pages,
    trap_handler::kernel_entry,
    utils::Addr,
};

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

    let paddr0 = alloc_pages(2).as_usize();
    let paddr1 = alloc_pages(1).as_usize();

    println!("alloc_pages test: paddr0 = {paddr0:x}");
    println!("alloc_pages test: paddr1 = {paddr1:x}");

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
