#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]
#![feature(custom_test_frameworks)]
#![test_runner(test::test_impl::test_runner)]
#![reexport_test_harness_main = "test_main"]
#![allow(clippy::missing_safety_doc)]

use alloc::string::ToString;
use core::{
    arch::{asm, global_asm},
    panic::PanicInfo,
};
use exit::{exit_qemu, QemuExitCode};
use multiboot::MultibootHeader;
use port::PortManager;

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
pub mod serial;
pub mod test;
pub mod time;
pub mod vga;

global_asm!(include_str!("boot.s"));

#[no_mangle]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub extern "C" fn kernel_main(magic: u32, multiboot_header: *const MultibootHeader) {
    let mut port_manager = PortManager::default();

    let mut interrupt_flag = interrupt::InterruptFlag::new();
    interrupt_guard!(interrupt_flag, {
        interrupt::init(&mut port_manager);
        gdt::init();
        idt::init();
    });

    let multiboot_header = unsafe { &*multiboot_header };
    multiboot::verify_mutliboot_magic(magic);
    multiboot::parse_multiboot_header(multiboot_header);
    memory::ALLOCATOR.init(multiboot_header);

    // Testing requires that the allocator be initialized, which requires the parsing the multiboot
    // header to find memory.
    #[cfg(test)]
    test_main();

    let cpu_features = cpuuid::get_cpu_features();
    assert!(cpu_features.contains(&cpuuid::CpuidFeatureEdx::APIC));

    let cmos = time::Cmos::new(&mut port_manager);
    let rtc = cmos.get_rtc();
    debug!("{:?}", rtc);

    interrupt!(0x80);

    // cmos.sleep(3);
    exit_qemu(QemuExitCode::Success);

    #[allow(clippy::empty_loop)]
    loop {}
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    match info.message().as_str() {
        Some(msg) => {
            error!("{}", msg)
        }
        None => {
            error!("Panic, aborting")
        }
    }
    if let Some(loc) = info.location() {
        error!(loc.to_string());
    }

    exit_qemu(QemuExitCode::Failed);
    loop {}
}
