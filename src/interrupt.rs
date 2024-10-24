use crate::{lock::spinlock::SpinLock, pic::Pic, warn};
use alloc::boxed::Box;
use core::fmt::Debug;
use lazy_static::lazy_static;

#[macro_export]
macro_rules! interrupt_guard {
    ($flag:ident, $blck:block) => {{
        let _guard = $flag.lock();
        let val = $blck;
        drop(_guard);
        val
    }};
}

#[macro_export]
macro_rules! interrupt {
    ($entry:tt) => {
        unsafe { core::arch::asm!(concat!("int ", stringify!($entry))) }
    };
}

// TODO: Make this model work concurrently.
//
// What happens when there are two processes that want to disable interrupts, one of them finishes
// and calls `sti`, which causes an interrupt for the other process?
pub struct InterruptGuard;

impl InterruptGuard {
    pub fn run<T>(f: impl FnOnce() -> T) -> T {
        unsafe {
            core::arch::asm!("cli");
            let ret = f();
            core::arch::asm!("sti");
            ret
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct InterruptFrame {
    pub ip: u32,
    pub cs: u32,
    pub flags: u32,
    pub sp: u32,
    pub ss: u32,
}

impl Debug for InterruptFrame {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let ip = self.ip;
        let cs = self.cs;
        let flags = self.flags;
        let sp = self.sp;
        let ss = self.ss;
        f.debug_struct("InterruptFrame")
            .field("ip", &format_args!("{:#x}", ip))
            .field("cs", &format_args!("{:#x}", cs))
            .field("flags", &format_args!("{:#x}", flags))
            .field("sp", &format_args!("{:#x}", sp))
            .field("ss", &format_args!("{:#x}", ss))
            .finish()
    }
}

pub fn interrupt_entry(irq: u8) {
    if let Some(handler) = INTERRUPT_LOOKUP.funcs.lock().get_mut(&irq) {
        handler.run();
    } else {
        warn!("interrupt {} not handled", irq);
    }
}

lazy_static! {
    pub static ref INTERRUPT_LOOKUP: InterruptLookup = InterruptLookup::default();
}

pub struct InterruptLookup {
    // TODO: This cannot be a spinlock because an interrupt may fire while registering
    funcs: SpinLock<hashbrown::HashMap<u8, InterruptHandler>>,
}

unsafe impl Send for InterruptLookup {}

impl Default for InterruptLookup {
    fn default() -> Self {
        Self {
            funcs: SpinLock::new(hashbrown::HashMap::default()),
        }
    }
}

impl InterruptLookup {
    pub fn register_handler(&self, handler: InterruptHandler) {
        let offset = match &handler {
            InterruptHandler::Exception(exc) => exc.vec_offset,
            InterruptHandler::Pic(pic) => pic.vec_offset() + Pic::VEC_OFFSET as u8,
        };

        self.funcs.lock().insert(offset, handler);
    }
}

pub enum InterruptHandler {
    Pic(PicHandler),
    Exception(ExceptionHandler),
}

impl InterruptHandler {
    pub fn run(&mut self) {
        match self {
            Self::Pic(pic) => (pic.func)(),
            Self::Exception(exc) => (exc.func)(),
        }
    }
}

pub struct PicHandler {
    pub irq_id: IrqId,
    pub func: Box<dyn FnMut()>,
}

impl PicHandler {
    pub fn new(irq_id: IrqId, func: impl FnMut() + 'static) -> Self {
        let func = Box::new(func);
        Self { irq_id, func }
    }

    pub fn vec_offset(&self) -> u8 {
        match self.irq_id {
            IrqId::Pic1(val) => val,
            IrqId::Pic2(val) => val + 8,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum IrqId {
    Pic1(u8),
    Pic2(u8),
}

pub struct ExceptionHandler {
    pub ty: ExceptionType,
    pub error_code: bool,
    pub func: Box<dyn Fn()>,
    pub vec_offset: u8,
}

impl ExceptionHandler {
    pub fn with_error_code(vec_offset: u8, ty: ExceptionType, func: impl Fn() + 'static) -> Self {
        debug_assert!(vec_offset < 32);
        let func = Box::new(func);
        Self {
            error_code: true,
            ty,
            vec_offset,
            func,
        }
    }

    pub fn without_error_code(
        vec_offset: u8,
        ty: ExceptionType,
        func: impl Fn() + 'static,
    ) -> Self {
        debug_assert!(vec_offset < 32);
        let func = Box::new(func);
        Self {
            error_code: false,
            ty,
            vec_offset,
            func,
        }
    }
}

pub enum ExceptionType {
    Fault,
    Trap,
    Abort,
}
