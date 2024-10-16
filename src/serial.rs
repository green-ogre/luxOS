use crate::{lock::spinlock::SpinLock, port::Port, println};
use lazy_static::lazy_static;

// https://os.phil-opp.com/testing/#printing-to-the-console
#[doc(hidden)]
pub fn _print(args: ::core::fmt::Arguments) {
    use core::fmt::Write;
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
    pub static ref SERIAL1: SpinLock<SerialPort> = {
        let serial_port = SerialPort::default();
        unsafe { serial_port.init() };
        SpinLock::new(serial_port)
    };
}

pub struct SerialPort {
    ports: [Port; 8],
}

impl core::fmt::Write for SerialPort {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        self.send_str(s.as_bytes());
        Ok(())
    }
}

impl Default for SerialPort {
    fn default() -> Self {
        unsafe {
            Self {
                ports: [
                    Port::new(0x3F8),
                    Port::new(0x3F9),
                    Port::new(0x3FA),
                    Port::new(0x3FB),
                    Port::new(0x3FC),
                    Port::new(0x3FD),
                    Port::new(0x3FE),
                    Port::new(0x3FF),
                ],
            }
        }
    }
}

impl SerialPort {
    unsafe fn init(&self) {
        // https://c9x.me/x86/html/file_module_x86_id_139.html
        self.ports[1].write(0x00); // Disable all interrupts
        self.ports[3].write(0x80); // Enable DLAB (set baud rate divisor)
        self.ports[0].write(0x03); // Set divisor to 3 (lo byte) 38400 baud
        self.ports[1].write(0x00); //                  (hi byte)
        self.ports[3].write(0x03); // 8 bits, no parity, one stop bit
        self.ports[2].write(0xC7); // Enable FIFO, clear them, with 14-byte threshold
        self.ports[4].write(0x0B); // IRQs enabled, RTS/DSR set
        self.ports[4].write(0x1E); // Set in loopback mode, test the serial chip
        self.ports[0].write(0xAE); // Test serial chip (send byte 0xAE and check if serial returns same byte)

        // Check if serial is faulty (i.e: not same byte as sent)
        if self.ports[0].read() != 0xAE {
            println!("Could not init serial port: 0x3F8");
            panic!();
        }

        // If serial is not faulty set it in normal operation mode
        // (not-loopback with IRQs enabled and OUT#1 and OUT#2 bits enabled)
        self.ports[4].write(0x0F);
    }

    fn is_transmit_empty(&self) -> u8 {
        unsafe { self.ports[5].read() & 0x20 }
    }

    fn send_str(&self, bytes: &[u8]) {
        for b in bytes.iter() {
            while self.is_transmit_empty() == 0 {}
            unsafe { self.ports[0].write(*b) };
        }
    }
}
