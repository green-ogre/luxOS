use crate::multiboot::MultibootHeader;

#[allow(unused)]
pub struct FrameBuffer {
    pixels: &'static mut [u32],
    pitch: u32,
    width: u32,
    height: u32,
    bpp: u8,
    ty: u8,
}

impl FrameBuffer {
    pub fn new(multiboot_header: &MultibootHeader) -> Self {
        assert!((multiboot_header.flags & (1 << 12)) > 0);
        assert_eq!(32, multiboot_header.framebuffer_bpp);

        let width = multiboot_header.framebuffer_width;
        let height = multiboot_header.framebuffer_height;

        let pixels = unsafe {
            core::slice::from_raw_parts_mut(
                multiboot_header.framebuffer_addr as *mut u64 as *mut u32,
                (width * height) as usize,
            )
        };

        Self {
            pixels,
            pitch: multiboot_header.framebuffer_pitch,
            bpp: multiboot_header.framebuffer_bpp,
            ty: multiboot_header.framebuffer_type,
            width,
            height,
        }
    }

    pub fn clear(&mut self) {
        for pixel in self.pixels.iter_mut() {
            *pixel = 0xFFFFFFFF;
        }
    }
}
