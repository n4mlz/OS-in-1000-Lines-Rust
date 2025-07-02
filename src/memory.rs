use core::{
    alloc::{GlobalAlloc, Layout},
    cell::RefCell,
    ptr,
};

use crate::{
    constants::{FREE_RAM, FREE_RAM_END, PAGE_SIZE},
    utils::{Addr, PhysAddr},
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
