use core::arch::asm;

pub fn init() {
    let size_of_gdt = 8 * 3;
    let gdt_addr = &GDT as *const GdtDescriptor as u64;
    let gdt_ptr = (size_of_gdt - 1) | (gdt_addr << 16);

    #[allow(named_asm_labels)]
    unsafe {
        asm!(
            "lgdt ({ptr})",
            "jmp $0x08, $.reload_segments",
            ".reload_segments:",
            "mov $0x10, {reg}",
            "mov {reg}, %ds",
            "mov {reg}, %es",
            "mov {reg}, %fs",
            "mov {reg}, %gs",
            "mov {reg}, %ss",
            ptr = in(reg) &gdt_ptr,
            reg = out(reg) _,
            options(att_syntax)
        )
    };
}

pub fn read_gdtr() -> u64 {
    let mut gdt = 0;
    unsafe {
        asm!("sgdt [{}]", in (reg) &mut gdt, options(nostack, preserves_flags));
    }
    gdt
}

static GDT: [GdtDescriptor; 3] = [
    GdtDescriptor::null(),
    GdtDescriptor::new(0, 0xFFFFF, Granularity::KiloBytes, true),
    GdtDescriptor::new(0, 0xFFFFF, Granularity::KiloBytes, false),
];

#[allow(unused)]
struct GdtDescriptor(u64);

#[allow(unused)]
impl GdtDescriptor {
    const PRESENT_BIT: u8 = 0x80;
    // Highest
    const PRIVILEGE: u8 = 0x00;
    const DESCRIPTOR_TYPE: u8 = 0x10;
    const ACCESSED: u8 = 0x1;

    pub const fn null() -> Self {
        Self(0)
    }

    pub const fn new(base: u32, limit: u32, granularity: Granularity, executable: bool) -> Self {
        if limit > 0xFFFFF {
            panic!("GDT cannot encode limits larger than 0xFFFFF");
        }

        let executable = if executable { 1 } else { 0 };
        let direction = 0;
        let read_write = 1;
        let access_byte = Self::PRESENT_BIT
            | Self::PRIVILEGE
            | Self::DESCRIPTOR_TYPE
            | (executable << 3)
            | (direction << 2)
            | (read_write << 1)
            | Self::ACCESSED;

        let granularity = match granularity {
            Granularity::Bytes => 0,
            Granularity::KiloBytes => 1,
        };
        let prot_32_mode = 1;
        let long_mode = 0;
        let flags = (granularity << 3) | (prot_32_mode << 2) | (long_mode << 1);

        let mut slf = Self::null();
        Self::write_entry(
            base,
            limit,
            access_byte,
            flags,
            &mut slf as *mut Self as *mut u8,
        );

        slf
    }

    fn base(&self) -> u32 {
        let mut base = self.bits(16, 24);
        let upper = self.bits(56, 8);
        base |= upper << 24;
        base as u32
    }

    fn limit(&self) -> u32 {
        let mut limit = self.bits(0, 16);
        let upper = self.bits(48, 4);
        limit |= upper << 16;
        limit as u32
    }

    fn access(&self) -> u8 {
        self.bits(40, 8) as u8
    }

    fn flags(&self) -> u8 {
        self.bits(52, 4) as u8
    }

    fn bits(&self, shift: usize, len: usize) -> u64 {
        let val = self.0;

        let mask = (1 << len) - 1;
        (val >> shift) & mask
    }

    // https://wiki.osdev.org/GDT_Tutorial
    const fn write_entry(base: u32, limit: u32, access_byte: u8, flags: u8, target: *mut u8) {
        unsafe {
            // Encode the limit
            *target = (limit & 0xFF) as u8;
            *target.add(1) = ((limit >> 8) & 0xFF) as u8;
            *target.add(6) = ((limit >> 16) & 0x0F) as u8;

            // Encode the base
            *target.add(2) = base as u8;
            *target.add(3) = (base >> 8) as u8;
            *target.add(4) = (base >> 16) as u8;
            *target.add(7) = (base >> 24) as u8;

            // Encode the access byte
            *target.add(5) = access_byte;

            // Encode the flags
            *target.add(6) |= flags << 4;
        }
    }
}

#[allow(unused)]
enum Granularity {
    Bytes,
    KiloBytes,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_case;

    test_case!(gdt_descriptors, {
        let gdt = GdtDescriptor::new(0xdeaf, 0xff33, Granularity::KiloBytes, false);
        test_assert_eq!(0xdeaf, gdt.base());
        test_assert_eq!(0xff33, gdt.limit());
        test_assert_eq!(0b1100, gdt.flags());
        let gdt = GdtDescriptor::new(0xff33, 0xdeaf, Granularity::Bytes, true);
        test_assert_eq!(0xff33, gdt.base());
        test_assert_eq!(0xdeaf, gdt.limit());
        test_assert_eq!(0b0100, gdt.flags());
    });
}
