unsafe extern "C" {
    static mut __bss: u8;
    static __bss_end: u8;
    static __stack_top: u8;
    static mut __free_ram: u8;
    static __free_ram_end: u8;
}

pub static mut BSS: *mut u8 = &raw mut __bss;
pub static mut BSS_END: *const u8 = &raw const __bss_end;
pub static mut STACK_TOP: *const u8 = &raw const __stack_top;
pub static mut FREE_RAM: *mut u8 = &raw mut __free_ram;
pub static mut FREE_RAM_END: *const u8 = &raw const __free_ram_end;

pub const PAGE_SIZE: usize = 4096;

pub const KERNEL_STACK_SIZE: usize = 8192;

pub const PROCS_MAX: u32 = 8;
