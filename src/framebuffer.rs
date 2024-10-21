use crate::multiboot::MultibootHeader;

static BACK_BUFFER: [u32; 786432] = [0; 786432];

#[allow(unused)]
pub struct FrameBuffer {
    pub width: usize,
    pub height: usize,
    front_buffer: &'static mut [u32],
    back_buffer: &'static mut [u32],
    pitch: usize,
    is_padding: bool,
    bpp: u8,
    ty: u8,
}

impl FrameBuffer {
    #[no_mangle]
    pub fn new(multiboot_header: &MultibootHeader) -> Self {
        assert!((multiboot_header.flags & (1 << 12)) > 0);
        assert_eq!(32, multiboot_header.framebuffer_bpp);

        let width = multiboot_header.framebuffer_width as usize;
        let height = multiboot_header.framebuffer_height as usize;
        let pitch = multiboot_header.framebuffer_pitch as usize;
        let bpp = multiboot_header.framebuffer_bpp as usize;

        let pixels_per_line = pitch / (bpp / 8);
        let is_padding = pixels_per_line != width;

        let front_buffer = unsafe {
            core::slice::from_raw_parts_mut(
                multiboot_header.framebuffer_addr as *mut u64 as *mut u32,
                pixels_per_line * height,
            )
        };

        let back_buffer = unsafe {
            core::slice::from_raw_parts_mut(
                core::ptr::addr_of!(BACK_BUFFER) as *mut u32,
                pixels_per_line * height,
            )
        };

        Self {
            front_buffer,
            back_buffer,
            bpp: multiboot_header.framebuffer_bpp,
            ty: multiboot_header.framebuffer_type,
            is_padding,
            width,
            height,
            pitch,
        }
    }

    pub fn present_frame(&mut self, f: impl FnOnce(&mut Self)) {
        self.clear(None);
        f(self);
        self.present();
    }

    pub fn draw_rect(&mut self, rect: &Rect) {
        let py = rect.tl.y;
        let px = rect.tl.x;

        for y in py..py + rect.dimensions.1 as isize {
            for x in px..px + rect.dimensions.0 as isize {
                if x < self.width as isize && y < self.height as isize && x >= 0 && y >= 0 {
                    self.back_buffer[y as usize * self.width + x as usize] = rect.color.as_u32();
                }
            }
        }
    }

    pub fn clear(&mut self, clear_color: Option<Color>) {
        if let Some(color) = clear_color {
            for pixel in self.back_buffer.iter_mut() {
                *pixel = color.as_u32();
            }
        } else {
            for pixel in self.back_buffer.iter_mut() {
                *pixel = 0x0;
            }
        }
    }

    pub fn present(&mut self) {
        let is_padding = self.pitch / (self.bpp as usize / 8) != self.width;

        if !is_padding {
            for i in 0..self.width * self.height {
                self.front_buffer[i] = self.back_buffer[i];
            }
        } else {
            let pitch = self.pitch / (self.bpp as usize / 8);
            for y in 0..self.height {
                for x in 0..self.width {
                    let fbi = y * pitch + x;
                    let bbi = y * self.width + x;

                    self.front_buffer[fbi] = self.back_buffer[bbi];
                }
            }
        }
    }
}

#[repr(C)]
pub struct Color {
    pub b: u8,
    pub g: u8,
    pub r: u8,
    pub a: u8,
}

impl Color {
    pub fn new_rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, a: 255 }
    }

    pub fn new_rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    pub fn as_u32(&self) -> u32 {
        unsafe { *(self as *const Color as *const u32) }
    }
}

pub struct Point {
    pub x: isize,
    pub y: isize,
}

impl Point {
    pub fn new(x: isize, y: isize) -> Self {
        Self { x, y }
    }

    fn index(&self, frame_buf: &FrameBuffer) -> Option<usize> {
        (self.x.is_positive() && self.y.is_positive())
            .then_some(self.y as usize * frame_buf.width + self.x as usize)
    }
}

pub struct Dimensions(usize, usize);

impl Dimensions {
    pub fn new(width: usize, height: usize) -> Self {
        Self(width, height)
    }
}

pub struct Rect {
    pub tl: Point,
    pub dimensions: Dimensions,
    pub color: Color,
}

impl Rect {
    pub fn new(top_left: Point, dimensions: Dimensions, color: Color) -> Self {
        Self {
            tl: top_left,
            dimensions,
            color,
        }
    }
}
