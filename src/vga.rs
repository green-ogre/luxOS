use crate::lock::spinlock::SpinLock;
use lazy_static::lazy_static;

lazy_static! {
    pub static ref WRITER: SpinLock<Writer> = SpinLock::new(Writer::default());
}

// #[macro_export]
// macro_rules! print {
//     () => {};
//     ($fmt:expr) => {
//         $crate::vga::WRITER.lock().write_str($fmt);
//     };
// }
//
// #[macro_export]
// macro_rules! println {
//     () => ($crate::print!(b"\n"));
//     ($fmt:expr) => ($crate::print!(concat!($fmt, "\n").as_bytes()));
//     ($($arg:tt)*) => {{
//         use core::fmt::Write;
//         write!($crate::vga::WRITER.lock(), "{}\n", format_args!($($arg)*)).unwrap();
//     }};
// }

const VGA_WIDTH: usize = 80;
const VGA_HEIGHT: usize = 25;

#[derive(Debug, Default, Clone, Copy)]
#[repr(C)]
struct VgaChar {
    unicode: u8,
    attributes: u8,
}

#[derive(Debug)]
struct Buffer {
    chars: [VgaChar; VGA_WIDTH * VGA_HEIGHT],
}

impl Default for Buffer {
    fn default() -> Self {
        Self {
            chars: [VgaChar::default(); VGA_WIDTH * VGA_HEIGHT],
        }
    }
}

impl Buffer {
    pub fn clear(&mut self) {
        for c in self.chars.iter_mut() {
            c.unicode = b' ';
            c.attributes = 0;
        }
    }

    pub fn clear_last_line(&mut self) {
        for c in self.chars.iter_mut().skip(VGA_WIDTH * (VGA_HEIGHT - 1)) {
            c.unicode = b' ';
            c.attributes = 0;
        }
    }
}

#[derive(Debug, Default)]
struct Cursor {
    x: usize,
    y: usize,
}

impl Cursor {
    pub fn index(&self) -> usize {
        self.y * VGA_WIDTH + self.x
    }

    pub fn next(&mut self) {
        self.x += 1;
        if self.x >= VGA_WIDTH {
            self.x = 0;
            self.y += 1;
        }
    }

    pub fn new_line(&mut self) {
        self.x = 0;
        self.y += 1;
    }
}

#[derive(Debug)]
pub struct Writer {
    cursor: Cursor,
    buffer: &'static mut Buffer,
}

impl core::fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        self.write_str(s.as_bytes());
        Ok(())
    }
}

impl Default for Writer {
    fn default() -> Self {
        let mut s = Self {
            cursor: Cursor::default(),
            buffer: unsafe { &mut *(0xb8000 as *mut Buffer) },
        };
        s.clear();

        s
    }
}

impl Writer {
    pub fn clear(&mut self) {
        self.buffer.clear();
    }

    pub fn write_byte(&mut self, byte: u8) {
        self.write_byte_unchecked(byte);
        self.check_and_handle_buffer();
    }

    pub fn write_str(&mut self, data: &[u8]) {
        data.iter().for_each(|c| self.write_byte(*c));
    }

    fn write_byte_unchecked(&mut self, byte: u8) {
        match byte {
            b'\n' => {
                self.cursor.new_line();
            }
            byte => {
                let index = self.cursor.index();
                if index >= VGA_HEIGHT * VGA_WIDTH {
                    panic!("overran vga buffer");
                }

                self.buffer.chars[index] = VgaChar {
                    unicode: byte,
                    attributes: 0x0F,
                };
                self.cursor.next();
            }
        }
    }

    fn check_and_handle_buffer(&mut self) {
        use core::cmp::Ordering;
        match self.cursor.y.cmp(&VGA_HEIGHT) {
            Ordering::Equal => {
                self.buffer.chars.as_mut_slice().copy_within(VGA_WIDTH.., 0);
                self.cursor.y -= 1;
                self.buffer.clear_last_line();
            }
            Ordering::Greater => {
                panic!("overran VGA buffer");
            }
            _ => {}
        }
    }
}
