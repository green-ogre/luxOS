use crate::{
    interrupt::IrqId,
    port::{Port, PortManager},
};

pub struct Pic {
    mmask: u8,
    smask: u8,

    mpic_cmd: Port,
    mpic_data: Port,

    spic_cmd: Port,
    spic_data: Port,
}

impl Pic {
    pub const VEC_OFFSET: usize = 0x20;

    pub fn new(port_manager: &mut PortManager) -> Self {
        let mpic_cmd = unsafe { port_manager.request_port(0x20).unwrap() };
        let mpic_data = unsafe { port_manager.request_port(0x21).unwrap() };
        let spic_cmd = unsafe { port_manager.request_port(0xA0).unwrap() };
        let spic_data = unsafe { port_manager.request_port(0xA1).unwrap() };

        let mmask = !(1 << 2);
        let smask = 0xFF;

        let mut slf = Self {
            mmask,
            smask,
            mpic_cmd,
            mpic_data,
            spic_cmd,
            spic_data,
        };
        slf.remap();

        slf
    }

    pub fn unmask(&mut self, index: IrqId) {
        match index {
            IrqId::Pic1(offset) => {
                // Prevent Pic2 from being masked
                if offset != 2 {
                    self.mmask &= !(1 << offset)
                }
            }
            IrqId::Pic2(offset) => self.smask &= !(1 << offset),
        }
        self.remap();
    }

    pub fn remap(&mut self) {
        const INIT: u8 = 0x10;
        const ICW4: u8 = 0x01;
        const IC8086: u8 = 0x01;

        unsafe {
            let master_mask = self.mmask;
            let slave_mask = self.smask;

            let master_offset = Self::VEC_OFFSET;
            let slave_offset = Self::VEC_OFFSET + 8;

            self.mpic_cmd.write(INIT | ICW4);
            self.spic_cmd.write(INIT | ICW4);
            self.mpic_data.write(master_offset as u8);
            self.spic_data.write(slave_offset as u8);

            self.mpic_data.write(4);
            self.spic_data.write(2);

            self.mpic_data.write(IC8086);
            self.spic_data.write(IC8086);

            self.mpic_data.write(master_mask);
            self.spic_data.write(slave_mask);
        }
    }
}
