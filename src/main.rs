#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]
#![feature(custom_test_frameworks)]
#![test_runner(test::test_runner)]
#![reexport_test_harness_main = "test_main"]
#![allow(clippy::missing_safety_doc)]

use alloc::string::ToString;
use core::{arch::global_asm, panic::PanicInfo};
use exit::{exit_qemu, QemuExitCode};
use multiboot::MultibootHeader;

extern crate alloc;

pub mod cpuuid;
pub mod exit;
pub mod framebuffer;
pub mod gdt;
pub mod idt;
pub mod interrupt;
pub mod kernel;
pub mod lock;
pub mod log;
pub mod memory;
pub mod multiboot;
pub mod pic;
pub mod port;
pub mod ps2;
pub mod serial;
pub mod test;
pub mod time;
pub mod vga;

global_asm!(include_str!("boot.s"));

#[no_mangle]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub extern "C" fn kernel_main(magic: u32, multiboot_header: *const MultibootHeader) {
    log::init(log::LogLevel::Info);

    let multiboot_header = unsafe { &*multiboot_header };
    multiboot::parse_multiboot_header(magic, multiboot_header);

    let mut kernel = kernel::Kernel::new(multiboot_header);
    info!("kernel initialized");

    // Testing requires that the allocator be initialized, which requires the parsing the multiboot
    // header to find memory.
    #[cfg(test)]
    test_main();

    kernel.run();
    // kernel.square_demo();
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    match info.message().as_str() {
        Some(msg) => {
            serial_println!("\nPANIC: {}", msg);

            // INFO: For some reason, running with cdrom in qemu will cause the `loc.to_string()`
            // call to core dump.

            // if let Some(loc) = info.location() {
            //     let val = loc.to_string();
            //     serial_println!("PANIC: {}", msg);
            // } else {
            //     serial_println!("PANIC: {}", msg);
            // }
        }
        None => {
            if let Some(loc) = info.location() {
                serial_println!("\nPANIC: {} => Panic, aborting", loc.to_string());
            } else {
                serial_println!("\nPANIC: Panic, aborting")
            }
        }
    }

    exit_qemu(QemuExitCode::Failed);
    loop {}
}
