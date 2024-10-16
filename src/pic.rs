use crate::port::PortManager;

pub fn init(port_manager: &mut PortManager) {
    unsafe { core::arch::asm!("cli") };

    let mpic_cmd = unsafe { port_manager.request_port(0x20).unwrap() };
    let mpic_data = unsafe { port_manager.request_port(0x21).unwrap() };
    let spic_cmd = unsafe { port_manager.request_port(0xA0).unwrap() };
    let spic_data = unsafe { port_manager.request_port(0xA1).unwrap() };

    unsafe {
        mpic_data.write(0xFF);
        spic_data.write(0xFF);
    }

    const INIT: u8 = 0x10;
    const ICW4: u8 = 0x01;
    const IC8086: u8 = 0x01;

    unsafe {
        let master_mask = mpic_data.read();
        let slave_mask = spic_data.read();

        let master_offset = 0x20;
        let slave_offset = 0x28;

        mpic_cmd.write(INIT | ICW4);
        spic_cmd.write(INIT | ICW4);
        mpic_data.write(master_offset);
        spic_data.write(slave_offset);

        mpic_data.write(4);
        spic_data.write(2);

        mpic_data.write(IC8086);
        spic_data.write(IC8086);

        mpic_data.write(master_mask);
        spic_data.write(slave_mask);
    }
}
