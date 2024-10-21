use crate::{
    framebuffer::*,
    gdt, idt,
    interrupt::{self, InterruptGuard, InterruptLookup},
    memory,
    multiboot::MultibootHeader,
    port::PortManager,
    ps2, serial, serial_println,
    time::Rtc,
};

#[allow(unused)]
pub struct Kernel {
    port_manager: PortManager,
    interrupt_lookup: &'static InterruptLookup,
    frame_buf: FrameBuffer,
}

fn check_interrupt_state(location: &str) {
    unsafe {
        let mut flags: u32;
        core::arch::asm!("pushf; pop {}", out(reg) flags);
        serial_println!(
            "Interrupt state at {}: IF={}",
            location,
            (flags & (1 << 9)) != 0
        );
    }
}

impl Kernel {
    pub fn new(multiboot_header: &MultibootHeader) -> Self {
        unsafe { core::arch::asm!("cli") };

        interrupt::InterruptGuard::run(|| {
            let mut port_manager = PortManager::default();
            gdt::init();
            let interrupt_lookup = idt::init();

            interrupt::init(&mut port_manager);
            memory::ALLOCATOR.init(multiboot_header);

            Rtc::enable_irq(&mut port_manager, interrupt_lookup);
            ps2::init(&mut port_manager, interrupt_lookup);

            interrupt_lookup.register_handler(interrupt::InterruptHandler::Pic(
                interrupt::PicHandler::new(interrupt::IrqId::Pic1(0), move || {
                    // Prevent PIT interrupt warnings
                }),
            ));

            let frame_buf = FrameBuffer::new(multiboot_header);

            Self {
                port_manager,
                interrupt_lookup,
                frame_buf,
            }
        })
    }

    pub fn run(&mut self) {
        #[allow(clippy::empty_loop)]
        loop {}
    }

    pub fn square_demo(&mut self) {
        let mut red_rect = Rect::new(
            Point::new(0, 500),
            Dimensions::new(200, 200),
            Color::new_rgb(255, 0, 0),
        );

        let mut yellow_rect = Rect::new(
            Point::new(400, 0),
            Dimensions::new(200, 200),
            Color::new_rgb(255, 255, 0),
        );

        // #[allow(clippy::empty_loop)]
        loop {
            red_rect.tl.x += 1;
            if red_rect.tl.x > 400 {
                red_rect.tl.x = 0;
            }

            yellow_rect.tl.y += 4;
            if yellow_rect.tl.y > 400 {
                yellow_rect.tl.y = 0;
            }

            self.frame_buf.present_frame(|frame: &mut FrameBuffer| {
                frame.draw_rect(&red_rect);
                frame.draw_rect(&yellow_rect);

                frame.draw_rect(&Rect::new(
                    Point::new(frame.width as isize - 100, 0),
                    Dimensions::new(200, 200),
                    Color::new_rgb(0, 255, 0),
                ));

                frame.draw_rect(&Rect::new(
                    Point::new(-100, -100),
                    Dimensions::new(200, 200),
                    Color::new_rgb(0, 0, 255),
                ));

                frame.draw_rect(&Rect::new(
                    Point::new(frame.width as isize - 100, frame.height as isize - 100),
                    Dimensions::new(200, 200),
                    Color::new_rgb(255, 0, 255),
                ));
            });
        }
    }
}
