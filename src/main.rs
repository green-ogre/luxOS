#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(test::test_runner)]
#![reexport_test_harness_main = "test_main"]
#![allow(clippy::missing_safety_doc)]

extern crate alloc;

use alloc::string::ToString;
use core::{arch::global_asm, panic::PanicInfo};
use exit::{exit_qemu, QemuExitCode};
use multiboot::MultibootHeader;

pub mod exit;
pub mod memory;
pub mod multiboot;
pub mod port;
pub mod serial;
pub mod test;
pub mod vga;

global_asm!(include_str!("boot.s"));

#[no_mangle]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub extern "C" fn kernel_main(magic: u32, multiboot_header: *const MultibootHeader) {
    let multiboot_header = unsafe { &*multiboot_header };
    multiboot::verify_mutliboot_magic(magic);
    multiboot::parse_multiboot_header(multiboot_header);
    memory::ALLOCATOR.init(multiboot_header);

    // Testing requires that the allocator be initialized, which requires the parsing the multiboot
    // header to find memory.
    #[cfg(test)]
    test_main();

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
