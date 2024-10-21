use crate::port::PortManager;

pub const PIC_VEC_OFFSET: usize = 0x20;

pub fn init(port_manager: &mut PortManager) {
    let mpic_cmd = unsafe { port_manager.request_port(0x20).unwrap() };
    let mpic_data = unsafe { port_manager.request_port(0x21).unwrap() };
    let spic_cmd = unsafe { port_manager.request_port(0xA0).unwrap() };
    let spic_data = unsafe { port_manager.request_port(0xA1).unwrap() };

    unsafe {
        mpic_data.write(0);
        spic_data.write(0);
    }

    const INIT: u8 = 0x10;
    const ICW4: u8 = 0x01;
    const IC8086: u8 = 0x01;

    unsafe {
        let master_mask = mpic_data.read();
        let slave_mask = spic_data.read();

        let master_offset = PIC_VEC_OFFSET;
        let slave_offset = PIC_VEC_OFFSET + 8;

        mpic_cmd.write(INIT | ICW4);
        spic_cmd.write(INIT | ICW4);
        mpic_data.write(master_offset as u8);
        spic_data.write(slave_offset as u8);

        mpic_data.write(4);
        spic_data.write(2);

        mpic_data.write(IC8086);
        spic_data.write(IC8086);

        mpic_data.write(master_mask);
        spic_data.write(slave_mask);

        // mpic_cmd.write(0x20);
        // spic_cmd.write(0x20);
    }
}
