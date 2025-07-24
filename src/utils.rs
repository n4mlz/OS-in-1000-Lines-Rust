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

#[macro_export]
macro_rules! write_csr_set {
    ($csr:literal, $val:expr) => {{
        asm!(concat!("csrs ", $csr, ", {}"), in(reg) $val);
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

pub trait Addr {
    const NULL: Self;
    fn from_usize(addr: usize) -> Self;
    fn from_ptr(addr: *const u8) -> Self;
    fn as_usize(&self) -> usize;
    fn as_ptr(&self) -> *const u8 {
        self.as_usize() as *const u8
    }
    fn as_ptr_mut(&self) -> *mut u8 {
        self.as_usize() as *mut u8
    }
    fn align_up(&self, align: usize) -> Self;
    fn is_aligned(&self, align: usize) -> bool;
}

macro_rules! impl_addr {
    ($name:ident) => {
        #[derive(Copy, Clone)]
        pub struct $name(usize);
        impl Addr for $name {
            const NULL: Self = $name(0);

            fn from_usize(addr: usize) -> Self {
                $name(addr)
            }

            fn from_ptr(addr: *const u8) -> Self {
                $name(addr as usize)
            }

            fn as_usize(&self) -> usize {
                self.0
            }

            fn align_up(&self, align: usize) -> Self {
                let offset = self.as_ptr().align_offset(align);
                $name::from_ptr(unsafe { self.as_ptr().add(offset) })
            }

            fn is_aligned(&self, align: usize) -> bool {
                self.as_ptr().is_aligned_to(align)
            }
        }
    };
}

impl_addr!(PhysAddr);
impl_addr!(VirtAddr);
