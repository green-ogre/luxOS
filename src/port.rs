use arrayvec::ArrayVec;
use core::arch::asm;

const RESERVED_PORTS: [u16; 9] = [
    0xF4, // Exit qemu
    // Serial IO
    0x3F8, 0x3F9, 0x3FA, 0x3FB, 0x3FC, 0x3FD, 0x3FE, 0x3FF,
];

/// Interface to port IO.
///
/// Each port must have only one active instance to prevent port clobbering.
///
/// [`https://wiki.osdev.org/Port_IO`]
#[derive(Debug, Default)]
pub struct PortManager {
    // TODO: Maybe this should be a Vec in the future?
    requested_ports: ArrayVec<u16, 32>,
}

impl PortManager {
    pub unsafe fn request_port(&mut self, port: u16) -> Option<Port> {
        debug_assert!(!RESERVED_PORTS.contains(&port));

        if !self.requested_ports.contains(&port) {
            self.requested_ports.push(port);
            Some(Port::new(port))
        } else {
            None
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
// TODO: On construct R/W permissions? Could be nice for Cmos.
pub struct Port(u16);

impl Port {
    pub unsafe fn new(port: u16) -> Self {
        Self(port)
    }

    pub unsafe fn write(&self, value: u8) {
        unsafe {
            asm!(
               "out dx, al",
               in("dx") self.0,
               in("al") value,
            );
        }
    }

    pub unsafe fn read(&self) -> u8 {
        let value: u8;
        unsafe {
            asm!(
                "in al, dx",
                in("dx") self.0,
                out("al") value,
            );
        }
        value
    }
}
