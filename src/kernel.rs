#![no_main]
#![no_std]
#![feature(fn_align, pointer_is_aligned_to)]

#[macro_use]
extern crate alloc;

mod apps;
mod constants;
mod ipc;
mod memory;
mod process;
mod sbi;
mod timer;
mod trap_handler;
mod utils;

use core::{arch::asm, fmt::Write, panic::PanicInfo, ptr};

use crate::{
    apps::{display, playground},
    constants::{BSS, BSS_END, STACK_TOP},
    memory::alloc_pages,
    process::PM,
    timer::init_timer,
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

    PM.create_process(display::display_server as usize);

    PM.create_process(playground::proc_a as usize);
    PM.create_process(playground::proc_b as usize);
    PM.create_process(playground::proc_c as usize);
    PM.create_process(playground::proc_d as usize);

    init_timer();

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
