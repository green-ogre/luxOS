use core::alloc::GlobalAlloc;

use crate::{println, serial_println};

#[repr(u32)]
#[derive(Debug, Clone, Copy)]
enum MmapType {
    Unknown = 0,
    Available = 1,
    Reserved = 2,
    AcpiReclaimable = 3,
    Nvs = 4,
    BadRam = 5,
}

#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
struct MmapEntry {
    size: u32,
    addr: u64,
    len: u64,
    ty: MmapType,
}

// https://wiki.osdev.org/Detecting_Memory_(x86)#Memory_Map_Via_GRUB
// TODO: This information _might_ be somewhere within memory, and not
// strictly confined to the first MB, meaning this needs to be saved.
struct MmapTable {
    // help: core::alloc::,
}

pub fn parse_mmap_table(mmap_length: usize, mmap_addr: u32) {
    let addr = mmap_addr as *const MmapEntry;
    serial_println!("{}", addr as u32);

    let entries = mmap_length / size_of::<MmapEntry>();
    serial_println!("Parsing {} mmap entries", entries);
    for i in 0..entries {
        let entry = unsafe { *addr.add(i) };
        serial_println!("{}: {:?}", i, entry);
    }
}

#[repr(align(4096))]
pub struct PageTable {
    entries: [PageTableEntry; 512],
}

#[repr(u16)]
enum PagePermissions {
    Read,
    Write,
    ReadWrite,
}

struct PageTableEntry {
    permissions: PagePermissions,
    index: u16,
    frame: u32,
}

#[derive(Default)]
pub struct Allocator {}

unsafe impl GlobalAlloc for Allocator {
    unsafe fn alloc(&self, layout: core::alloc::Layout) -> *mut u8 {
        self.alloc(layout)
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: core::alloc::Layout) {
        self.dealloc(ptr, layout);
    }
}

impl Allocator {
    pub const fn new() -> Self {
        Self {}
    }

    unsafe fn alloc(&self, layout: core::alloc::Layout) -> *mut u8 {
        println!("Allocating: {layout:?}");
        core::ptr::NonNull::dangling().as_ptr()
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: core::alloc::Layout) {
        println!("Deallocating ptr: {:#x} : {layout:?}", ptr as u64);
    }
}
