use crate::{interrupt::enable_interrupts, serial_println};
use core::{arch::asm, cell::RefCell};

pub fn init() {
    init_idt();
    let size_of_idt = size_of::<u64>() as u64 * 256;
    let idt_addr = IDT.entries.as_ptr() as *const GateDescriptor as u64;
    serial_println!("idt size: {}", size_of_idt);
    serial_println!("init idt addr: {:#x}", idt_addr);
    let idt_ptr = (size_of_idt - 1) | (idt_addr << 16);

    #[allow(named_asm_labels)]
    unsafe {
        asm!(
            "lidt ({ptr})",
            ptr = in(reg) &idt_ptr,
            options(att_syntax)
        )
    };
    enable_interrupts();
}

pub fn read_idtr() -> u64 {
    let mut gdt = 0;
    unsafe {
        asm!("sidt [{}]", in (reg) &mut gdt, options(nostack, preserves_flags));
    }
    gdt
}

const NUM_GATE_DESC: usize = 256;

static IDT: InterruptTable = InterruptTable::new();

struct InterruptTable {
    entries: RefCell<[GateDescriptor; NUM_GATE_DESC]>,
}

unsafe impl Sync for InterruptTable {}

impl InterruptTable {
    pub const fn new() -> Self {
        Self {
            entries: RefCell::new([GateDescriptor::null(); NUM_GATE_DESC]),
        }
    }

    pub fn set_entry(&self, descriptor: GateDescriptor, entry: usize) {
        debug_assert!(entry < NUM_GATE_DESC);
        self.entries.borrow_mut()[entry] = descriptor;
    }
}

fn init_idt() {
    IDT.set_entry(
        #[allow(clippy::fn_to_numeric_cast)]
        GateDescriptor::new(
            crate::interrupt::general_fault_handler as u32,
            SegmentSelector::GDT_CODE,
            GateType::Interrupt32,
        ),
        0x80,
    );
}

#[derive(Debug, Clone, Copy)]
struct GateDescriptor(u64);

#[allow(unused)]
impl GateDescriptor {
    pub const fn null() -> Self {
        Self(0)
    }

    pub const fn new(isr_offset: u32, selector: SegmentSelector, gate_type: GateType) -> Self {
        let dpl = 0;

        let mut slf = Self::null();
        Self::write_entry(
            isr_offset,
            selector,
            gate_type,
            dpl,
            &mut slf as *mut Self as *mut u8,
        );

        slf
    }

    fn offset(&self) -> u32 {
        let mut base = self.bits(0, 16);
        let upper = self.bits(48, 16);
        base |= upper << 16;
        base as u32
    }

    fn selector(&self) -> u16 {
        self.bits(16, 16) as u16
    }

    fn gate_type(&self) -> u8 {
        self.bits(40, 4) as u8
    }

    fn dpl(&self) -> u8 {
        self.bits(45, 2) as u8
    }

    fn present(&self) -> bool {
        self.bits(47, 1) == 1
    }

    fn bits(&self, shift: usize, len: usize) -> u64 {
        let val = self.0;

        let mask = (1 << len) - 1;
        (val >> shift) & mask
    }

    /// https://wiki.osdev.org/Interrupt_Descriptor_Table
    const fn write_entry(
        isr_offset: u32,
        selector: SegmentSelector,
        gate_type: GateType,
        dpl: u8,
        target: *mut u8,
    ) {
        unsafe {
            *target = (isr_offset & 0xFF) as u8;
            *target.add(1) = ((isr_offset >> 8) & 0xFF) as u8;

            *target.add(2) = (selector.encoded_value() & 0xFF) as u8;
            *target.add(3) = ((selector.encoded_value() >> 8) & 0xFF) as u8;

            *target.add(5) = gate_type.value() | (dpl << 5) | (1 << 7);

            *target.add(6) = ((isr_offset >> 16) & 0xFF) as u8;
            *target.add(7) = ((isr_offset >> 24) & 0xFF) as u8;
        }
    }
}

/// https://wiki.osdev.org/Segment_Selector
#[derive(Debug, Clone, Copy)]
struct SegmentSelector(u16);

#[allow(unused)]
impl SegmentSelector {
    pub const GDT_DATA: Self = Self::new(GdtIndex::Data);
    pub const GDT_CODE: Self = Self::new(GdtIndex::Code);

    pub const fn new(index: GdtIndex) -> Self {
        Self(index.value() << 3)
    }

    pub const fn encoded_value(&self) -> u16 {
        self.0
    }
}

#[derive(Debug, Clone, Copy)]
#[allow(unused)]
enum GdtIndex {
    Code,
    Data,
}

impl GdtIndex {
    pub const fn value(&self) -> u16 {
        match self {
            Self::Code => 1,
            Self::Data => 2,
        }
    }
}

#[derive(Debug, Clone, Copy)]
#[allow(unused)]
enum GateType {
    Task,
    Interrupt16,
    Trap16,
    Interrupt32,
    Trap32,
}

impl GateType {
    pub const fn value(&self) -> u8 {
        match self {
            Self::Task => 0b0101,
            Self::Interrupt16 => 0b0110,
            Self::Trap16 => 0b0111,
            Self::Interrupt32 => 0b1110,
            Self::Trap32 => 0b1111,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_descriptor(isr_offset: u32, selector: SegmentSelector, gate_type: GateType) {
        let gate = GateDescriptor::new(isr_offset, selector, gate_type);
        serial_println!("{:#x}, {:#x}", gate.offset(), isr_offset);
        assert_eq!(gate.offset(), isr_offset);
        assert_eq!(gate.selector(), selector.encoded_value());
        assert_eq!(gate.gate_type(), gate_type.value());
        assert!(gate.present());
    }

    #[test_case]
    fn idt_descriptors() {
        test_descriptor(0, SegmentSelector::GDT_DATA, GateType::Interrupt32);
        test_descriptor(0xdeafdeaf, SegmentSelector::GDT_DATA, GateType::Task);
        test_descriptor(0xd2203122, SegmentSelector::GDT_DATA, GateType::Task);

        let idt_addr = (read_idtr() >> 16) as *const u64;
        serial_println!(
            "int 0x80: {:#b}, {:#x}",
            unsafe { *idt_addr.add(0x80) },
            IDT.entries.borrow()[0x80].0
        );
        assert_eq!(unsafe { *idt_addr.add(0x80) }, IDT.entries.borrow()[0x80].0);
    }
}
