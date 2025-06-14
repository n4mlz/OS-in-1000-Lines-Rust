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
