use core::{
    alloc::GlobalAlloc,
    fmt::Debug,
    sync::atomic::{AtomicPtr, Ordering},
};

use crate::{multiboot::MultibootHeader, serial_println};

#[global_allocator]
pub static ALLOCATOR: Allocator = Allocator::new();

#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
pub struct Allocator {
    pub first_header: AtomicPtr<u8>,
}

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
        Self {
            // This is safe because nothing in the program can interact with the GlobalAllocator
            // before the kernel initializes it.
            first_header: AtomicPtr::new(core::ptr::null_mut()),
        }
    }

    /// Parses the mmap to find the largest chunk of contiguous memory for the first header.
    pub fn init(&self, multiboot_header: &MultibootHeader) {
        let mmap_length = multiboot_header.mmap_length as usize;
        let mmap_addr = multiboot_header.mmap_addr;

        let addr = mmap_addr as *const MmapEntry;
        serial_println!("{}", addr as u32);

        let entries = mmap_length / size_of::<MmapEntry>();
        serial_println!("Parsing {} mmap entries", entries);
        let mut largest_entry = 0;
        let mut largest_memory_size = 0;
        for i in 0..entries {
            let entry = unsafe { *addr.add(i) };
            let memory_size = entry.len;
            let entry_ty = entry.ty;
            if entry_ty == MmapType::Available && memory_size > largest_memory_size {
                largest_memory_size = memory_size;
                largest_entry = i;
            }
            serial_println!("{}: {:?}", i, entry);
        }

        if largest_memory_size == 0 {
            panic!("No available memory on system, what the fuck?");
        }

        let entry = unsafe { *addr.add(largest_entry) };
        serial_println!("largest entry: {}\n{:?}", largest_entry, entry);
        // TODO: figure out how the ordering works
        //
        // https://marabos.nl/atomics/
        self.first_header
            .store(entry.addr as *mut u8, Ordering::Relaxed);
        serial_println!(
            "allocator start: {}",
            self.first_header.load(Ordering::Relaxed) as u64,
        );

        let first_header = entry.addr as *mut AllocHeader;
        unsafe {
            *first_header = AllocHeader {
                // Size of header does not matter in this case
                len: entry.len as u32,
                // Null pointer
                next_header_addr: 0,
            }
        }
    }

    unsafe fn alloc(&self, layout: core::alloc::Layout) -> *mut u8 {
        // Again, need to figure out ordering
        let first_header_ptr = self.first_header.load(Ordering::Relaxed);
        let mut current_header = *(first_header_ptr as *mut AllocHeader);
        let mut current_address = first_header_ptr;
        let header_padding = size_of::<AllocHeader>() % layout.align();
        let mut target_len = layout.size() + header_padding + size_of::<AllocHeader>();
        // Want to make sure that everything is always 4 byte aligned
        let end_padding = target_len % 4;
        if end_padding != 0 {
            target_len += 4 - end_padding;
        }

        serial_println!("\ncurrent_address: {:#x}", current_address as u64);
        serial_println!("header_padding: {}", header_padding);
        serial_println!("layout: {:?}", layout);
        serial_println!("target_len: {}", target_len);

        loop {
            if current_header.is_occupied() {
                serial_println!("\tcurrent header is occupied");
                if !current_header.next_header_is_valid() {
                    panic!("\t\tcurrent header points to null next header");
                } else {
                    current_address = current_address.add(current_header.len() as usize);
                    serial_println!(
                        "\t\tcurrent header points to non-null next header, jumping to {:#x}...",
                        current_address as u64
                    );
                    current_header = *(current_address as *mut AllocHeader);

                    continue;
                }
            }

            // Actually do the allocation
            if !current_header.is_occupied() && current_header.len() as usize >= target_len {
                serial_println!(
                    "\tcurrent header is not occupied and the size >= target_len, breaking..."
                );

                let previous_len = current_header.len();

                current_header.set_occupied();
                let alloc_ptr = current_address.add(header_padding + size_of::<AllocHeader>());
                serial_println!(
                    "\talloc_ptr: {:#x}, distance from header start: {}",
                    alloc_ptr as u64,
                    alloc_ptr as u64 - current_address as u64
                );
                debug_assert!(current_header.is_occupied());

                // Create a new header if necessary
                if !current_header.next_header_is_valid() {
                    current_header.set_len(target_len as u32);
                    let next_header_len = previous_len - target_len as u32;
                    *(current_address.add(target_len) as *mut AllocHeader) =
                        AllocHeader::new(next_header_len);
                    current_header.next_header_addr = current_address.add(target_len) as u32;
                    serial_println!("\t\tsetting next header: {:?}", current_header);
                }

                *(current_address as *mut AllocHeader) = current_header;

                return alloc_ptr;
            }

            if !current_header.next_header_is_valid() {
                serial_println!("\tcurrent header points to null next header, breaking...");
                break;
            } else {
                current_address = current_header.next_header_addr as *mut u8;
                serial_println!(
                    "\tcurrent header points to non-null next header, jumping to {:#x}...",
                    current_address as u64
                );
                current_header = *(current_address as *mut AllocHeader);
            }
        }

        panic!("buy more ram nerd");
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: core::alloc::Layout) {
        serial_println!("\nDeallocating ptr: {:#x} : {:?}", ptr as u64, layout);
        let first_header_ptr = self.first_header.load(Ordering::Relaxed);
        let mut current_header = *(first_header_ptr as *mut AllocHeader);
        let mut current_address = first_header_ptr;

        loop {
            if !current_header.next_header_is_valid()
                && current_header.len() + (current_address as u32) < ptr as u32
            {
                panic!("where was this allocated?");
            }

            // Actually do the deallocation
            if (current_address as u32) < ptr as u32 && current_header.next_header_addr > ptr as u32
            {
                // TODO: verify this pointer?
                current_header.set_vacant();
                *(current_address as *mut AllocHeader) = current_header;
                debug_assert!(!current_header.is_occupied());

                return;
            } else {
                current_address = current_header.next_header_addr as *mut u8;
                current_header = *(current_address as *mut AllocHeader);
            }
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct AllocHeader {
    /// Includes the header size
    /// Occupation is packed into len
    len: u32,
    next_header_addr: u32,
}

impl Debug for AllocHeader {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("AllocHeader")
            .field("len", &self.len())
            .field("is_occupied", &self.is_occupied())
            .field(
                "next_header_addr",
                &format_args!("{:#x}", self.next_header_addr),
            )
            .finish()
    }
}

impl AllocHeader {
    pub fn new(len: u32) -> Self {
        Self {
            len: len << 1,
            next_header_addr: 0,
        }
    }

    pub fn is_occupied(&self) -> bool {
        self.len & 1 > 0
    }

    pub fn set_occupied(&mut self) {
        self.len |= 1;
    }

    pub fn set_vacant(&mut self) {
        self.len &= !1;
    }

    pub fn len(&self) -> u32 {
        self.len >> 1
    }

    pub fn set_len(&mut self, len: u32) {
        debug_assert!(len == ((len << 1) >> 1));
        self.len = (self.len & 1) | (len << 1);
    }

    pub fn next_header_is_valid(&self) -> bool {
        self.next_header_addr != 0
    }

    pub fn next_header_addr(&self) -> u32 {
        self.next_header_addr
    }
}
