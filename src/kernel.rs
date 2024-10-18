use crate::{
    framebuffer::FrameBuffer,
    gdt, idt,
    interrupt::{self, InterruptGuard, InterruptLookup},
    memory,
    multiboot::MultibootHeader,
    port::PortManager,
    ps2,
    time::Rtc,
};

#[allow(unused)]
pub struct Kernel {
    port_manager: PortManager,
    interrupt_lookup: &'static InterruptLookup,
    frame_buf: FrameBuffer,
}

impl Kernel {
    pub fn new(multiboot_header: &MultibootHeader) -> Self {
        InterruptGuard::run(|| {
            let mut port_manager = PortManager::default();
            interrupt::init(&mut port_manager);

            gdt::init();
            let interrupt_lookup = idt::init();

            memory::ALLOCATOR.init(multiboot_header);

            Rtc::enable_irq(&mut port_manager, interrupt_lookup);
            ps2::init(&mut port_manager, interrupt_lookup);

            let frame_buf = FrameBuffer::new(multiboot_header);

            Self {
                port_manager,
                interrupt_lookup,
                frame_buf,
            }
        })
    }

    pub fn run(&mut self) {
        self.frame_buf.clear();

        #[allow(clippy::empty_loop)]
        loop {}
    }
}
