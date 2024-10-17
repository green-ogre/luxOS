#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]
#![feature(custom_test_frameworks)]
#![test_runner(test::test_impl::test_runner)]
#![reexport_test_harness_main = "test_main"]
#![allow(clippy::missing_safety_doc)]

use alloc::string::ToString;
use core::{arch::global_asm, panic::PanicInfo};
use exit::{exit_qemu, QemuExitCode};
use interrupt::{InterruptHandler, IrqId, PicHandler};
use multiboot::MultibootHeader;
use port::PortManager;
use time::Rtc;

extern crate alloc;

pub mod cpuuid;
pub mod exit;
pub mod gdt;
pub mod idt;
pub mod interrupt;
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
    let mut port_manager = PortManager::default();

    let interrupt_lookup;
    let mut interrupt_flag = interrupt::InterruptFlag::new();
    interrupt_guard!(interrupt_flag, {
        interrupt::init(&mut port_manager);
        gdt::init();
        interrupt_lookup = idt::init();
    });
    multiboot::parse_multiboot_header(magic, multiboot_header);
    memory::ALLOCATOR.init(multiboot_header);

    // Testing requires that the allocator be initialized, which requires the parsing the multiboot
    // header to find memory.
    #[cfg(test)]
    test_main();

    let cpu_features = cpuuid::get_cpu_features();
    assert!(cpu_features.contains(&cpuuid::CpuidFeatureEdx::APIC));

    Rtc::enable_irq(&mut port_manager, &mut interrupt_flag, interrupt_lookup);
    ps2::init(&mut port_manager, &mut interrupt_flag, interrupt_lookup);

    // interrupt_guard!(interrupt_flag, {
    //     interrupt_lookup.register_handler(InterruptHandler::Pic(PicHandler::new(
    //         IrqId::Pic1(0),
    //         move || {},
    //     )));
    // });

    info!("lux initialized");

    #[allow(clippy::empty_loop)]
    loop {}
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    serial_println!();
    match info.message().as_str() {
        Some(msg) => {
            if let Some(loc) = info.location() {
                serial_println!("PANIC: {} => {}", loc.to_string(), msg);
            } else {
                serial_println!("PANIC: {}", msg);
            }
        }
        None => {
            if let Some(loc) = info.location() {
                serial_println!("PANIC: {} => Panic, aborting", loc.to_string());
            } else {
                serial_println!("PANIC: Panic, aborting")
            }
        }
    }

    exit_qemu(QemuExitCode::Failed);
    loop {}
}
