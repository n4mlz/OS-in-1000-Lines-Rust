use core::fmt::{self, Write};

use crate::sbi::sbi_call;

#[macro_export]
macro_rules! read_csr {
    ($csr:literal) => {{
        let mut val: usize;
        unsafe {
            core::arch::asm!(concat!("csrr {}, ", $csr), out(reg) val);
        }
        val
    }};
}

#[macro_export]
macro_rules! write_csr {
    ($csr:literal, $val:expr) => {{
        asm!(concat!("csrw ", $csr, ", {}"), in(reg) $val);
    }};
}

pub fn putchar(c: u8) -> Result<(), isize> {
    sbi_call(c as usize, 0, 0, 0, 0, 0, 0, 1)?;

    Ok(())
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ({
        let _ = write!( $crate::utils::Writer, $($arg)*);
    });
}

#[macro_export]
macro_rules! println {
    ($($arg:tt)*) => ({
        print!("{}\n", format_args!($($arg)*));
    });
}

pub struct Writer;

impl Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for c in s.bytes() {
            putchar(c).map_err(|_| fmt::Error)?;
        }
        Ok(())
    }
}

pub fn align_up(value: usize, align: usize) -> usize {
    let r = value % align;
    if r == 0 { value } else { value + (align - r) }
}

pub trait Addr {
    fn get(&self) -> usize;
    fn from_usize(addr: usize) -> Self;
    fn from_ptr(addr: *const u8) -> Self;
    fn as_usize(&self) -> usize {
        self.get()
    }
    fn as_ptr(&self) -> *const u8 {
        self.get() as *const u8
    }
    fn as_ptr_mut(&self) -> *mut u8 {
        self.get() as *mut u8
    }
    fn align_up(&self, align: usize) -> Self;
}

macro_rules! impl_addr {
    ($name:ident) => {
        #[derive(Copy, Clone)]
        pub struct $name(usize);
        impl Addr for $name {
            fn get(&self) -> usize {
                self.0
            }

            fn from_usize(addr: usize) -> Self {
                $name(addr)
            }

            fn from_ptr(addr: *const u8) -> Self {
                $name(addr as usize)
            }

            fn align_up(&self, align: usize) -> Self {
                $name(align_up(self.get(), align))
            }
        }
    };
}

impl_addr!(PhysAddr);
impl_addr!(VirtAddr);
