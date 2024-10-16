#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]
#![feature(custom_test_frameworks)]
#![test_runner(test::test_runner)]
#![reexport_test_harness_main = "test_main"]
#![allow(clippy::missing_safety_doc)]

extern crate alloc;

use alloc::{
    string::{String, ToString},
    vec::Vec,
};
use core::{
    arch::{asm, global_asm},
    panic::PanicInfo,
};
use exit::{exit_qemu, QemuExitCode};
use multiboot::MultibootHeader;
use port::PortManager;

pub mod exit;
pub mod gdt;
pub mod idt;
pub mod interrupt;
pub mod memory;
pub mod multiboot;
pub mod pic;
pub mod port;
pub mod serial;
pub mod test;
pub mod time;
pub mod vga;

global_asm!(include_str!("boot.s"));

#[no_mangle]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub extern "C" fn kernel_main(magic: u32, multiboot_header: *const MultibootHeader) {
    let mut port_manager = PortManager::default();
    pic::init(&mut port_manager);

    gdt::init();
    let gdtr = gdt::read_gdtr();
    serial_println!("post gdt_addr: {:#x}", gdtr >> 16);

    idt::init();
    let idtr = idt::read_idtr();
    serial_println!("post idt_addr: {:#x}", idtr >> 16);

    interrupt!(0x80);

    let multiboot_header = unsafe { &*multiboot_header };
    multiboot::verify_mutliboot_magic(magic);
    multiboot::parse_multiboot_header(multiboot_header);
    memory::ALLOCATOR.init(multiboot_header);

    // Testing requires that the allocator be initialized, which requires the parsing the multiboot
    // header to find memory.
    #[cfg(test)]
    test_main();

    let cpu_features = get_cpu_features();
    assert!(cpu_features.contains(&CpuidFeatureEdx::APIC));

    let cmos = time::Cmos::new(&mut port_manager);
    let rtc = cmos.get_rtc();
    serial_println!("{:?}", rtc);

    // cmos.sleep(3);
    exit_qemu(QemuExitCode::Success);

    #[allow(clippy::empty_loop)]
    loop {}
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    match info.message().as_str() {
        Some(msg) => serial_println!(msg),
        None => serial_println!("Panic, aborting"),
    }
    if let Some(loc) = info.location() {
        serial_println!(loc.to_string());
    }

    exit_qemu(QemuExitCode::Failed);
    loop {}
}

#[repr(u32)]
#[allow(non_camel_case_types, clippy::upper_case_acronyms, unused)]
/// Only found within newer chips?
enum CpuidFeatureEcx {
    SSE3 = 1 << 0,
    PCLMUL = 1 << 1,
    DTES64 = 1 << 2,
    MONITOR = 1 << 3,
    DS_CPL = 1 << 4,
    VMX = 1 << 5,
    SMX = 1 << 6,
    EST = 1 << 7,
    TM2 = 1 << 8,
    SSSE3 = 1 << 9,
    CID = 1 << 10,
    SDBG = 1 << 11,
    FMA = 1 << 12,
    CX16 = 1 << 13,
    XTPR = 1 << 14,
    PDCM = 1 << 15,
    PCID = 1 << 17,
    DCA = 1 << 18,
    SSE4_1 = 1 << 19,
    SSE4_2 = 1 << 20,
    X2APIC = 1 << 21,
    MOVBE = 1 << 22,
    POPCNT = 1 << 23,
    TSC = 1 << 24,
    AES = 1 << 25,
    XSAVE = 1 << 26,
    OSXSAVE = 1 << 27,
    AVX = 1 << 28,
    F16C = 1 << 29,
    RDRAND = 1 << 30,
    HYPERVISOR = 1 << 31,
}

#[repr(u32)]
#[derive(Debug, PartialEq, Eq)]
#[allow(non_camel_case_types, clippy::upper_case_acronyms, unused)]
enum CpuidFeatureEdx {
    FPU = 1 << 0,
    VME = 1 << 1,
    DE = 1 << 2,
    PSE = 1 << 3,
    TSC = 1 << 4,
    MSR = 1 << 5,
    PAE = 1 << 6,
    MCE = 1 << 7,
    CX8 = 1 << 8,
    APIC = 1 << 9,
    SEP = 1 << 11,
    MTRR = 1 << 12,
    PGE = 1 << 13,
    MCA = 1 << 14,
    CMOV = 1 << 15,
    PAT = 1 << 16,
    PSE36 = 1 << 17,
    PSN = 1 << 18,
    CLFLUSH = 1 << 19,
    DS = 1 << 21,
    ACPI = 1 << 22,
    MMX = 1 << 23,
    FXSR = 1 << 24,
    SSE = 1 << 25,
    SSE2 = 1 << 26,
    SS = 1 << 27,
    HTT = 1 << 28,
    TM = 1 << 29,
    IA64 = 1 << 30,
    PBE = 1 << 31,
}

fn get_cpu_features() -> Vec<CpuidFeatureEdx> {
    let b: u32;
    let d: u32;
    let c: u32;

    // Verify vendor id
    unsafe {
        asm!(
            "mov eax, 0x0",
            "cpuid",
            out("ebx") b,
            out("edx") d,
            out("ecx") c,
        );
    }

    let mut vendor_id = String::new();

    let mut push_reg = |reg: u32| {
        vendor_id.push((reg & 0xFF) as u8 as char);
        vendor_id.push(((reg >> 8) & 0xFF) as u8 as char);
        vendor_id.push(((reg >> 16) & 0xFF) as u8 as char);
        vendor_id.push(((reg >> 24) & 0xFF) as u8 as char);
    };

    push_reg(b);
    push_reg(d);
    push_reg(c);

    serial_println!("CPU vendor id: {}", vendor_id);
    assert_eq!("GenuineIntel".to_string(), vendor_id);

    // Gather cpu features
    let d: u32;
    unsafe {
        asm!(
            "mov eax, 0x1",
            "cpuid",
            out("edx") d,
        );
    }

    let mut features: Vec<CpuidFeatureEdx> = Vec::with_capacity(32);
    serial_println!("cpuid: {:#b}", d);
    for i in 0..32 {
        // TODO: match statement
        let feature = (d >> i) & 1;
        if feature != 0 {
            #[allow(clippy::missing_transmute_annotations)]
            features.push(unsafe { core::mem::transmute(1 << i) });
        }
    }
    serial_println!("features: {:?}", features);
    features
}
