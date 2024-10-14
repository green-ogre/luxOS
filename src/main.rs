#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(test_runner)]
#![reexport_test_harness_main = "test_main"]
#![allow(clippy::missing_safety_doc)]

use core::{arch::global_asm, panic::PanicInfo};
use exit::{exit_qemu, QemuExitCode};
use multiboot::MultibootHeader;

pub mod exit;
pub mod memory;
pub mod multiboot;
pub mod serial;
pub mod vga;

global_asm!(include_str!("boot.s"));

#[global_allocator]
static ALLOCATOR: memory::Allocator = memory::Allocator::new();

#[no_mangle]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub extern "C" fn kernel_main(magic: u32, multiboot_header: *const MultibootHeader) {
    #[cfg(test)]
    test_main();

    if magic != 0x2BADB002 {
        panic!("Invalid mutliloader magic");
    }

    let multiboot_header = unsafe { &*multiboot_header };
    serial_println!("flags: {:#b}", multiboot_header.flags);
    serial_println!(
        "mem_lower: {}, present: {}",
        multiboot_header.mem_lower,
        (multiboot_header.flags & (1 << 0)) > 0
    );
    serial_println!(
        "mem_upper: {}, present: {}",
        multiboot_header.mem_upper,
        (multiboot_header.flags & (1 << 0)) > 0
    );
    serial_println!(
        "boot_device: {}, present: {}",
        multiboot_header.boot_device,
        (multiboot_header.flags & (1 << 1)) > 0
    );
    serial_println!(
        "cmdline: {}, present: {}",
        multiboot_header.cmdline,
        (multiboot_header.flags & (1 << 2)) > 0
    );
    serial_println!(
        "mods_count: {}, present: {}",
        multiboot_header.mods_count,
        (multiboot_header.flags & (1 << 3)) > 0
    );
    serial_println!(
        "mods_addr: {}, present: {}",
        multiboot_header.mods_addr,
        (multiboot_header.flags & (1 << 3)) > 0
    );
    serial_println!(
        "syms1: {}, present: {}",
        multiboot_header.syms1,
        (multiboot_header.flags & (1 << 4)) > 0
    );
    serial_println!(
        "syms2: {}, present: {}",
        multiboot_header.syms2,
        (multiboot_header.flags & (1 << 4)) > 0
    );
    serial_println!(
        "syms3: {}, present: {}",
        multiboot_header.syms3,
        (multiboot_header.flags & (1 << 4)) > 0
    );
    serial_println!(
        "mmap_length: {}, present: {}",
        multiboot_header.mmap_length,
        (multiboot_header.flags & (1 << 6)) > 0
    );
    serial_println!(
        "mmap_addr: {}, present: {}",
        multiboot_header.mmap_addr,
        (multiboot_header.flags & (1 << 6)) > 0
    );
    serial_println!(
        "drives_length: {}, present: {}",
        multiboot_header.drives_length,
        (multiboot_header.flags & (1 << 7)) > 0
    );
    serial_println!(
        "drives_addr: {}, present: {}",
        multiboot_header.drives_addr,
        (multiboot_header.flags & (1 << 7)) > 0
    );
    serial_println!(
        "config_table: {}, present: {}",
        multiboot_header.config_table,
        (multiboot_header.flags & (1 << 8)) > 0
    );
    serial_println!(
        "boot_loader_name: {}, present: {}",
        multiboot_header.boot_loader_name,
        (multiboot_header.flags & (1 << 9)) > 0
    );
    serial_println!(
        "apm_table: {}, present: {}",
        multiboot_header.apm_table,
        (multiboot_header.flags & (1 << 10)) > 0
    );

    if (multiboot_header.flags & (1 << 9)) > 0 {
        let boot_loader_name = multiboot_header.boot_loader_name as *const u8;
        let name = unsafe { core::ffi::CStr::from_ptr(boot_loader_name as *const i8) };
        if let Ok(name_str) = name.to_str() {
            serial_println!("Boot loader name: {}", name_str);
        }
    }

    let memory_range = multiboot_header.mem_upper - multiboot_header.mem_lower;
    serial_println!("Memory range: {}mb", memory_range / 1024);

    serial_println!();
    assert!((multiboot_header.flags & (1 << 6)) > 0);
    memory::parse_mmap_table(
        multiboot_header.mmap_length as usize,
        multiboot_header.mmap_addr,
    );

    #[allow(clippy::empty_loop)]
    loop {}
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    match info.message().as_str() {
        Some(msg) => serial_println!(msg),
        None => serial_println!("Panic, aborting"),
    }
    exit_qemu(QemuExitCode::Failed);
    loop {}
}

#[cfg(test)]
pub fn test_runner(tests: &[&dyn Fn()]) {
    serial_println!("Running {} tests\n", tests.len());
    for test in tests {
        test();
    }
    serial_println!("\nAll tests passed...");
    exit_qemu(QemuExitCode::Success);
}

#[test_case]
fn trivial_assertion() {
    serial_println!("trivial assertion... ");
    serial_println!("[ok]");
}
