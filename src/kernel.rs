use crate::{
    framebuffer::*,
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

    pub fn run(&mut self) {}

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
