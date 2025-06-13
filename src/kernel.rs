#![no_main]
#![no_std]

use core::{
    arch::asm,
    fmt::{self, Write},
    panic::PanicInfo,
    ptr,
};

unsafe extern "C" {
    static mut __bss: u8;
    static __bss_end: u8;
    static __stack_top: u8;
}

#[allow(clippy::too_many_arguments)]
fn sbi_call(
    arg0: usize,
    arg1: usize,
    arg2: usize,
    arg3: usize,
    arg4: usize,
    arg5: usize,
    fid: usize,
    eid: usize,
) -> Result<usize, isize> {
    let mut err: isize;
    let mut value: usize;

    unsafe {
        asm!(
            "ecall",
            inout("a0") arg0 => err,
            inout("a1") arg1 => value,
            in("a2") arg2,
            in("a3") arg3,
            in("a4") arg4,
            in("a5") arg5,
            in("a6") fid,
            in("a7") eid,
        );
    }

    if err < 0 { Err(err) } else { Ok(value) }
}

fn putchar(c: u8) -> Result<(), isize> {
    sbi_call(c as usize, 0, 0, 0, 0, 0, 0, 1)?;

    Ok(())
}

macro_rules! print {
    ($($arg:tt)*) => ({
        let _ = write!(Writer, $($arg)*);
    });
}

macro_rules! println {
    ($($arg:tt)*) => ({
        print!("{}\n", format_args!($($arg)*));
    });
}

struct Writer;

impl Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for c in s.bytes() {
            putchar(c).map_err(|_| fmt::Error)?;
        }
        Ok(())
    }
}

#[unsafe(no_mangle)]
#[unsafe(link_section = ".text.boot")]
extern "C" fn boot() -> ! {
    unsafe {
        asm!(
            "mv sp, {stack_top}
            j {kernel_main}",
            stack_top = in(reg) &__stack_top,
            kernel_main = sym kernel_main,
            options(noreturn)
        );
    }
}

fn kernel_main() -> ! {
    unsafe {
        let bss = ptr::addr_of_mut!(__bss);
        let bss_end = ptr::addr_of!(__bss_end);
        ptr::write_bytes(bss, 0, bss_end as usize - bss as usize);
    }

    println!("Hello, World!");
    panic!("Kernel panic: This is a test panic!");

    loop {}
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
