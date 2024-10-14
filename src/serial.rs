use crate::println;
use core::arch::asm;
use core::fmt::Write;
use lazy_static::lazy_static;
use spin::Mutex;

// https://os.phil-opp.com/testing/#printing-to-the-console
#[doc(hidden)]
pub fn _print(args: ::core::fmt::Arguments) {
    SERIAL1
        .lock()
        .write_fmt(args)
        .expect("Printing to serial failed");
}

/// Prints to the host through the serial interface.
#[macro_export]
macro_rules! serial_print {
    ($($arg:tt)*) => {
        $crate::serial::_print(format_args!($($arg)*))
    };
}

/// Prints to the host through the serial interface, appending a newline.
#[macro_export]
macro_rules! serial_println {
    () => {
        $crate::serial_print!("\n")
    };
    ($fmt:expr) => {
        $crate::serial_print!("{}\n", $fmt)
    };
    ($fmt:expr, $($arg:tt)*) => {
        $crate::serial_print!(concat!($fmt, "\n"), $($arg)*)
    };
}

lazy_static! {
    pub static ref SERIAL1: Mutex<SerialPort> = {
        let serial_port = SerialPort::new(0x3F8);
        serial_port.init();
        Mutex::new(serial_port)
    };
}

pub struct SerialPort {
    port: u32,
}

impl core::fmt::Write for SerialPort {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        self.send_str(s.as_bytes());
        Ok(())
    }
}

impl SerialPort {
    pub fn new(port: u32) -> Self {
        Self { port }
    }

    pub fn writeb(&self, byte: u8) {
        outb(byte, self.port);
    }

    pub fn readb(&self) -> u8 {
        inb(self.port)
    }

    fn init(&self) {
        // https://c9x.me/x86/html/file_module_x86_id_139.html
        outb(0x00, self.port + 1); // Disable all interrupts
        outb(0x80, self.port + 3); // Enable DLAB (set baud rate divisor)
        outb(0x03, self.port); // Set divisor to 3 (lo byte) 38400 baud
        outb(0x00, self.port + 1); //                  (hi byte)
        outb(0x03, self.port + 3); // 8 bits, no parity, one stop bit
        outb(0xC7, self.port + 2); // Enable FIFO, clear them, with 14-byte threshold
        outb(0x0B, self.port + 4); // IRQs enabled, RTS/DSR set
        outb(0x1E, self.port + 4); // Set in loopback mode, test the serial chip
        outb(0xAE, self.port); // Test serial chip (send byte 0xAE and check if serial returns same byte)

        // Check if serial is faulty (i.e: not same byte as sent)
        if inb(self.port) != 0xAE {
            println!("Could not init serial port: {}", self.port);
            panic!();
        }

        // If serial is not faulty set it in normal operation mode
        // (not-loopback with IRQs enabled and OUT#1 and OUT#2 bits enabled)
        outb(0x0F, self.port + 4);
    }

    fn is_transmit_empty(&self) -> u8 {
        inb(self.port + 5) & 0x20
    }

    fn send_str(&self, bytes: &[u8]) {
        for b in bytes.iter() {
            while self.is_transmit_empty() == 0 {}
            outb(*b, self.port);
        }
    }
}

fn outb(value: u8, port: u32) {
    unsafe {
        asm!(
           "out dx, al",
           in("dx") port,
           in("al") value,
        );
    }
}

fn inb(port: u32) -> u8 {
    let value: u8;
    unsafe {
        asm!(
            "in al, dx",
            in("dx") port,
            out("al") value,
        );
    }
    value
}
