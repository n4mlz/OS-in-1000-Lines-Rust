#![no_main]
#![no_std]
#![feature(fn_align, pointer_is_aligned_to)]

mod constants;
mod memory;
mod playground;
mod process;
mod sbi;
mod trap_handler;
mod utils;

use core::{arch::asm, fmt::Write, panic::PanicInfo, ptr};

use crate::{
    constants::{BSS, BSS_END, STACK_TOP},
    memory::alloc_pages,
    process::PM,
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

    PM.init();

    println!("Hello, World!");

    let paddr0 = alloc_pages(2).as_usize();
    let paddr1 = alloc_pages(1).as_usize();

    println!("alloc_pages test: paddr0 = {paddr0:x}");
    println!("alloc_pages test: paddr1 = {paddr1:x}");

    PM.crate_process(playground::proc_a_entry as usize);
    PM.crate_process(playground::proc_b_entry as usize);

    PM.switch();

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
