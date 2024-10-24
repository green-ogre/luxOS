use crate::port::{PortManager, PortSlice};

pub struct SerialPort {
    ports: PortSlice<8>,
}

impl core::fmt::Write for SerialPort {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        self.send_str(s.as_bytes());
        Ok(())
    }
}

impl SerialPort {
    pub fn new(port_manager: &mut PortManager) -> Self {
        unsafe {
            Self {
                ports: port_manager
                    .request_range(0x3F8)
                    .expect("could not acquire serial ports"),
            }
        }
    }

    pub unsafe fn init(&self) {
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
            panic!("failed to init serial port");
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
            self.write_byte(*b);
        }
    }

    pub fn write_byte(&self, byte: u8) {
        while self.is_transmit_empty() == 0 {
            unsafe { crate::port::Port::new(0x3F8).write(b'A') };
        }
        unsafe { self.ports[0].write(byte) };
    }
}
