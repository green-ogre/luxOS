use arrayvec::ArrayVec;
use core::{arch::asm, ops::Deref};

const RESERVED_PORTS: [u16; 1] = [
    0xF4, // Exit qemu
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

    pub unsafe fn request_range<const LEN: usize>(
        &mut self,
        offset: u16,
    ) -> Option<PortSlice<LEN>> {
        assert!(
            !offset.overflowing_add(LEN as u16).1,
            "port range is too large"
        );

        let mut slice = PortSlice(ArrayVec::new());
        for port in offset..offset + LEN as u16 {
            if let Some(port) = self.request_port(port) {
                slice.0.push(port);
            } else {
                return None;
            }
        }
        debug_assert!(slice.0.len() == LEN);
        Some(slice)
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

pub struct PortSlice<const LEN: usize>(ArrayVec<Port, LEN>);

impl<const LEN: usize> Deref for PortSlice<LEN> {
    type Target = ArrayVec<Port, LEN>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
