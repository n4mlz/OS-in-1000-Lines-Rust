use core::{
    alloc::{GlobalAlloc, Layout},
    cell::RefCell,
    ptr,
};

use crate::{
    constants::{FREE_RAM, FREE_RAM_END, PAGE_SIZE, PAGE_V},
    utils::{Addr, PhysAddr, VirtAddr},
};

struct Alocator {
    head: RefCell<*mut u8>,
    end: *const u8,
}

impl Alocator {
    const fn new(start: *mut u8, end: *const u8) -> Self {
        Alocator {
            head: RefCell::new(start),
            end,
        }
    }
}

unsafe impl Sync for Alocator {}

unsafe impl GlobalAlloc for Alocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let head = *self.head.borrow();

        let size = layout.size();
        let align = layout.align();

        let padding = head.align_offset(align);
        let alloc_start = unsafe { head.add(padding) };
        let alloc_end = unsafe { alloc_start.add(size) };

        if alloc_end as *const u8 > self.end {
            ptr::null_mut()
        } else {
            unsafe { ptr::write_bytes(alloc_start, 0, size) };

            let mut head = self.head.borrow_mut();
            *head = alloc_end;

            alloc_start
        }
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {}
}

#[global_allocator]
static HEAP: Alocator = unsafe { Alocator::new(FREE_RAM, FREE_RAM_END) };

pub fn alloc_pages(num: usize) -> PhysAddr {
    let size = num * PAGE_SIZE;

    let layout = match Layout::from_size_align(size, PAGE_SIZE) {
        Ok(layout) => layout,
        Err(_) => {
            panic!("Invalid layout for allocation");
        }
    };

    let ptr = unsafe { HEAP.alloc(layout) };
    if ptr.is_null() {
        panic!("Out of memory");
    }

    PhysAddr::from_ptr(ptr)
}

pub fn map_page(page_table: PhysAddr, vaddr: VirtAddr, paddr: PhysAddr, flags: u32) {
    if !vaddr.is_aligned(PAGE_SIZE) || !paddr.is_aligned(PAGE_SIZE) {
        panic!("Virtual and physical addresses must be page-aligned");
    }

    let table1 = page_table.as_usize() as *mut u32;
    let vpn1 = ((vaddr.as_usize() >> 22) & 0x3ff) as isize;

    if unsafe { *table1.offset(vpn1) } & PAGE_V == 0 {
        let pt_paddr = alloc_pages(1);
        unsafe { *table1.offset(vpn1) = ((pt_paddr.as_usize() / PAGE_SIZE) << 10) as u32 | PAGE_V };
    }

    let table0 = ((unsafe { *table1.offset(vpn1) } >> 10) * PAGE_SIZE as u32) as *mut u32;
    let vpn0 = ((vaddr.as_usize() >> 12) & 0x3ff) as isize;

    unsafe {
        *(table0.offset(vpn0)) = ((paddr.as_usize() / PAGE_SIZE) << 10) as u32 | flags | PAGE_V
    };
}
