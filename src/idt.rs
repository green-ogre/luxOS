use crate::{
    interrupt::{InterruptFrame, InterruptLookup, INTERRUPT_LOOKUP},
    serial_println,
};
use core::{arch::asm, cell::RefCell};

const NUM_GATE_DESC: usize = 256;

pub fn init() -> &'static InterruptLookup {
    init_idt();
    let size_of_idt = size_of::<u64>() as u64 * NUM_GATE_DESC as u64;
    let idt_addr = IDT.entries.as_ptr() as *const GateDescriptor as u64;
    let idt_ptr = (size_of_idt - 1) | (idt_addr << 16);

    #[allow(named_asm_labels)]
    unsafe {
        asm!(
            "lidt ({ptr})",
            ptr = in(reg) &idt_ptr,
            options(att_syntax)
        )
    };

    &INTERRUPT_LOOKUP
}

macro_rules! register_default_handler {
    ($entry:expr) => {
        IDT.set_entry(
            GateDescriptor::new(
                #[allow(clippy::fn_to_numeric_cast)]
                {
                    paste::paste! {
                        #[no_mangle]
                        #[allow(clippy::not_unsafe_ptr_arg_deref)]
                        pub extern "x86-interrupt" fn [<__default_handler $entry>] (_frame: $crate::interrupt::InterruptFrame) {
                            $crate::interrupt::interrupt_entry($entry);
                        }

                        [<__default_handler $entry>] as u32
                    }
                },
                SegmentSelector::GDT_CODE,
                GateType::Interrupt32,
            ),
            $entry,
        );
    };
}

macro_rules! register_handlers {
    ($($n:expr),*) => {
        $(
            register_default_handler!($n);
        )*
    };
}

macro_rules! register_err_code_handler {
    ($entry:expr) => {
        IDT.set_entry(
            GateDescriptor::new(
                #[allow(clippy::fn_to_numeric_cast)]
                {
                    paste::paste! {
                        #[no_mangle]
                        #[allow(clippy::not_unsafe_ptr_arg_deref)]
                        pub extern "x86-interrupt" fn [<__default_handler $entry>] (_frame: $crate::interrupt::InterruptFrame, _err: u32) {
                            $crate::interrupt::interrupt_entry($entry);
                        }

                        [<__default_handler $entry>] as u32
                    }
                },
                SegmentSelector::GDT_CODE,
                GateType::Interrupt32,
            ),
            $entry,
        );
    };
}

macro_rules! register_err_code_handlers {
    ($($n:expr),*) => {
        $(
            register_err_code_handler!($n);
        )*
    };
}

macro_rules! register_pic_handler {
    ($entry:expr) => {
        IDT.set_entry(
            GateDescriptor::new(
                #[allow(clippy::fn_to_numeric_cast)]
                {
                    paste::paste! {
                        #[no_mangle]
                        #[allow(clippy::not_unsafe_ptr_arg_deref)]
                        pub extern "x86-interrupt" fn [<__default_handler $entry>] (_frame: $crate::interrupt::InterruptFrame) {
                            $crate::interrupt::interrupt_entry($entry);
                            unsafe {
                                let pic1 = $crate::port::Port::new(0x20);
                                let pic2 = $crate::port::Port::new(0xA0);
                                if ($entry >= 8 + $crate::pic::PIC_VEC_OFFSET) {
                                    pic2.write(0x20);
                                }
                                pic1.write(0x20);
                            }
                        }

                        [<__default_handler $entry>] as u32
                    }
                },
                SegmentSelector::GDT_CODE,
                GateType::Interrupt32,
            ),
            $entry,
        );
    };
}

macro_rules! register_pic_handlers {
    ($($n:expr),*) => {
        $(
            register_pic_handler!($n);
        )*
    };
}

fn init_idt() {
    register_handlers!(
        0, 1, 2, 3, 4, 5, 6, 7, 9, 15, 16, 19, 20, 22, 23, 24, 25, 26, 27, 28, 31, 48, 49, 50, 51,
        52, 53, 54, 55, 56, 57, 58, 59, 60, 61, 62, 63, 64, 65, 66, 67, 68, 69, 70, 71, 72, 73, 74,
        75, 76, 77, 78, 79, 80, 81, 82, 83, 84, 85, 86, 87, 88, 89, 90, 91, 92, 93, 94, 95, 96, 97,
        98, 99, 100, 101, 102, 103, 104, 105, 106, 107, 108, 109, 110, 111, 112, 113, 114, 115,
        116, 117, 118, 119, 120, 121, 122, 123, 124, 125, 126, 127, 128, 129, 130, 131, 132, 133,
        134, 135, 136, 137, 138, 139, 140, 141, 142, 143, 144, 145, 146, 147, 148, 149, 150, 151,
        152, 153, 154, 155, 156, 157, 158, 159, 160, 161, 162, 163, 164, 165, 166, 167, 168, 169,
        170, 171, 172, 173, 174, 175, 176, 177, 178, 179, 180, 181, 182, 183, 184, 185, 186, 187,
        188, 189, 190, 191, 192, 193, 194, 195, 196, 197, 198, 199, 200, 201, 202, 203, 204, 205,
        206, 207, 208, 209, 210, 211, 212, 213, 214, 215, 216, 217, 218, 219, 220, 221, 222, 223,
        224, 225, 226, 227, 228, 229, 230, 231, 232, 233, 234, 235, 236, 237, 238, 239, 240, 241,
        242, 243, 244, 245, 246, 247, 248, 249, 250, 251, 252, 253, 254, 255
    );

    register_err_code_handlers!(10, 11, 12, 13, 14, 17, 21, 29, 30);

    register_pic_handlers!(
        32, // 33,
        34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45, 46, 47
    );

    IDT.set_entry(
        GateDescriptor::new(
            #[allow(clippy::fn_to_numeric_cast)]
            {
                #[no_mangle]
                #[allow(clippy::not_unsafe_ptr_arg_deref)]
                pub extern "x86-interrupt" fn double_fault(_frame: InterruptFrame, _err: u32) {
                    panic!("double fault");
                }

                double_fault as u32
            },
            SegmentSelector::GDT_CODE,
            GateType::Interrupt32,
        ),
        8,
    );

    IDT.set_entry(
        GateDescriptor::new(
            #[allow(clippy::fn_to_numeric_cast)]
            {
                #[no_mangle]
                #[allow(clippy::not_unsafe_ptr_arg_deref)]
                pub extern "x86-interrupt" fn machine_check(_frame: InterruptFrame, _err: u32) {
                    panic!("machine check");
                }

                machine_check as u32
            },
            SegmentSelector::GDT_CODE,
            GateType::Interrupt32,
        ),
        18,
    );

    IDT.set_entry(
        GateDescriptor::new(
            #[allow(clippy::fn_to_numeric_cast)]
            {
                #[no_mangle]
                #[allow(clippy::not_unsafe_ptr_arg_deref)]
                pub extern "x86-interrupt" fn ps2(_frame: InterruptFrame) {
                    unsafe {
                        let data = crate::port::Port::new(0x60);
                        let result = data.read();
                        serial_println!("{:#x}", result);

                        let pic1 = crate::port::Port::new(0x20);
                        pic1.write(0x20);
                    }
                }

                ps2 as u32
            },
            SegmentSelector::GDT_CODE,
            GateType::Interrupt32,
        ),
        33,
    );
}

static IDT: InterruptTable = InterruptTable::new();

pub struct InterruptTable {
    entries: RefCell<[GateDescriptor; NUM_GATE_DESC]>,
}

unsafe impl Sync for InterruptTable {}

impl InterruptTable {
    #[allow(clippy::new_without_default)]
    pub const fn new() -> Self {
        Self {
            entries: RefCell::new([GateDescriptor::null(); NUM_GATE_DESC]),
        }
    }

    fn set_entry(&self, descriptor: GateDescriptor, entry: usize) {
        debug_assert!(entry < NUM_GATE_DESC);
        self.entries.borrow_mut()[entry] = descriptor;
    }
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
    use crate::{debug, test::test_impl::TestResult, test_assert, test_assert_eq, test_case};

    fn test_descriptor(
        isr_offset: u32,
        selector: SegmentSelector,
        gate_type: GateType,
    ) -> TestResult {
        let gate = GateDescriptor::new(isr_offset, selector, gate_type);
        debug!("{:#x}, {:#x}", gate.offset(), isr_offset);
        test_assert_eq!(gate.offset(), isr_offset);
        test_assert_eq!(gate.selector(), selector.encoded_value());
        test_assert_eq!(gate.gate_type(), gate_type.value());
        test_assert!(gate.present());
        TestResult::Success
    }

    test_case!(idt_descriptors, {
        test_descriptor(0, SegmentSelector::GDT_DATA, GateType::Interrupt32);
        test_descriptor(0xdeafdeaf, SegmentSelector::GDT_DATA, GateType::Task);
        test_descriptor(0xd2203122, SegmentSelector::GDT_DATA, GateType::Task);

        let idt_addr = (read_idtr() >> 16) as *const u64;
        debug!(
            "int 0x80: {:#b}, {:#x}",
            unsafe { *idt_addr.add(0x80) },
            IDT.entries.borrow()[0x80].0
        );
        test_assert_eq!(unsafe { *idt_addr.add(0x80) }, IDT.entries.borrow()[0x80].0);
    });
}
