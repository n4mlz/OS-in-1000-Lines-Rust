#![no_main]
#![no_std]
#![feature(fn_align)]

mod sbi;
mod trap_handler;
mod utils;

use crate::trap_handler::kernel_entry;
use core::{arch::asm, fmt::Write, panic::PanicInfo, ptr};

unsafe extern "C" {
    static mut __bss: u8;
    static __bss_end: u8;
    static __stack_top: u8;
}

static mut BSS: *mut u8 = ptr::addr_of_mut!(__bss);
static mut BSS_END: *const u8 = ptr::addr_of!(__bss_end);
static mut STACK_TOP: *const u8 = ptr::addr_of!(__stack_top);

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
