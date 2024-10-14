use core::arch::asm;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum QemuExitCode {
    Success = 0x10,
    Failed = 0x11,
}

pub fn exit_qemu(exit_code: QemuExitCode) {
    let mut port = Port::new(0xf4);
    port.write(exit_code as u32);
}

struct Port {
    port: u16,
}

impl Port {
    pub fn new(port: u16) -> Self {
        Self { port }
    }

    pub fn write(&mut self, value: u32) {
        outb(value, self.port);
    }
}

fn outb(value: u32, port: u16) {
    unsafe {
        asm!(
           "out dx, eax",
           in("dx") port,
           in("eax") value,
        );
    }
}
