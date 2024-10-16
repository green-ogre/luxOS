use crate::serial_println;
use core::arch::asm;

pub fn enable_interrupts() {
    unsafe { asm!("sti") };
}

pub fn disable_interrupts() {
    unsafe { asm!("cli") };
}

#[macro_export]
macro_rules! interrupt {
    ($entry:tt) => {
        unsafe { asm!(concat!("int ", stringify!($entry))) }
    };
}

#[no_mangle]
pub extern "x86-interrupt" fn general_fault_handler() {
    serial_println!("INT: general_fault_handler");
}
