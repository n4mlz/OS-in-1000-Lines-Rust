#![no_main]
#![no_std]
#![feature(fn_align)]

use core::{
    arch::{asm, naked_asm},
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

#[unsafe(naked)]
#[repr(align(4))]
#[unsafe(no_mangle)]
unsafe extern "C" fn kernel_entry() {
    naked_asm!(
        "
        csrw sscratch, sp
        addi sp, sp, -4 * 31
        sw ra,  4 * 0(sp)
        sw gp,  4 * 1(sp)
        sw tp,  4 * 2(sp)
        sw t0,  4 * 3(sp)
        sw t1,  4 * 4(sp)
        sw t2,  4 * 5(sp)
        sw t3,  4 * 6(sp)
        sw t4,  4 * 7(sp)
        sw t5,  4 * 8(sp)
        sw t6,  4 * 9(sp)
        sw a0,  4 * 10(sp)
        sw a1,  4 * 11(sp)
        sw a2,  4 * 12(sp)
        sw a3,  4 * 13(sp)
        sw a4,  4 * 14(sp)
        sw a5,  4 * 15(sp)
        sw a6,  4 * 16(sp)
        sw a7,  4 * 17(sp)
        sw s0,  4 * 18(sp)
        sw s1,  4 * 19(sp)
        sw s2,  4 * 20(sp)
        sw s3,  4 * 21(sp)
        sw s4,  4 * 22(sp)
        sw s5,  4 * 23(sp)
        sw s6,  4 * 24(sp)
        sw s7,  4 * 25(sp)
        sw s8,  4 * 26(sp)
        sw s9,  4 * 27(sp)
        sw s10, 4 * 28(sp)
        sw s11, 4 * 29(sp)

        csrr a0, sscratch
        sw a0, 4 * 30(sp)

        mv a0, sp
        call handle_trap

        lw ra,  4 * 0(sp)
        lw gp,  4 * 1(sp)
        lw tp,  4 * 2(sp)
        lw t0,  4 * 3(sp)
        lw t1,  4 * 4(sp)
        lw t2,  4 * 5(sp)
        lw t3,  4 * 6(sp)
        lw t4,  4 * 7(sp)
        lw t5,  4 * 8(sp)
        lw t6,  4 * 9(sp)
        lw a0,  4 * 10(sp)
        lw a1,  4 * 11(sp)
        lw a2,  4 * 12(sp)
        lw a3,  4 * 13(sp)
        lw a4,  4 * 14(sp)
        lw a5,  4 * 15(sp)
        lw a6,  4 * 16(sp)
        lw a7,  4 * 17(sp)
        lw s0,  4 * 18(sp)
        lw s1,  4 * 19(sp)
        lw s2,  4 * 20(sp)
        lw s3,  4 * 21(sp)
        lw s4,  4 * 22(sp)
        lw s5,  4 * 23(sp)
        lw s6,  4 * 24(sp)
        lw s7,  4 * 25(sp)
        lw s8,  4 * 26(sp)
        lw s9,  4 * 27(sp)
        lw s10, 4 * 28(sp)
        lw s11, 4 * 29(sp)
        lw sp,  4 * 30(sp)
        sret
        ",
    );
}

#[repr(C, packed)]
pub struct TrapFrame {
    ra: usize,
    gp: usize,
    tp: usize,
    t0: usize,
    t1: usize,
    t2: usize,
    t3: usize,
    t4: usize,
    t5: usize,
    t6: usize,
    a0: usize,
    a1: usize,
    a2: usize,
    a3: usize,
    a4: usize,
    a5: usize,
    a6: usize,
    a7: usize,
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
    sp: usize,
}

macro_rules! read_csr {
    ($csr:literal) => {{
        let mut val: u32;
        unsafe {
            asm!(concat!("csrr {}, ", $csr), out(reg) val);
        }
        val
    }};
}

macro_rules! write_csr {
    ($csr:literal, $val:expr) => {{
        asm!(concat!("csrw ", $csr, ", {}"), in(reg) $val);
    }};
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
