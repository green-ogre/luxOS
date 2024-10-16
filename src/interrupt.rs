use crate::{info, port::PortManager};

pub fn init(port_manager: &mut PortManager) {
    crate::pic::init(port_manager);
}

// TODO: Make this model work concurrently.
//
// What happens when there are two processes that want to disable interrupts, one of them finishes
// and calls `sti`, which causes an interrupt for the other process?
pub struct InterruptFlag;

impl InterruptFlag {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self
    }

    pub fn lock(&mut self) -> InterruptGuard<'_> {
        InterruptGuard::new(self)
    }
}

#[allow(unused)]
pub struct InterruptGuard<'a>(&'a mut InterruptFlag);

impl<'a> InterruptGuard<'a> {
    pub fn new(flag: &'a mut InterruptFlag) -> Self {
        unsafe { core::arch::asm!("cli") };
        Self(flag)
    }
}

impl Drop for InterruptGuard<'_> {
    fn drop(&mut self) {
        unsafe { core::arch::asm!("sti") };
    }
}

#[macro_export]
macro_rules! interrupt_guard {
    ($flag:ident, $blck:block) => {
        let _guard = $flag.lock();
        $blck
        drop(_guard);
    }
}

#[macro_export]
macro_rules! interrupt {
    ($entry:tt) => {
        unsafe { asm!(concat!("int ", stringify!($entry))) }
    };
}

#[no_mangle]
pub extern "x86-interrupt" fn general_fault_handler() {
    info!("INT: general_fault_handler");
}
