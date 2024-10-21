use crate::{
    info,
    interrupt::{InterruptHandler, InterruptLookup, IrqId, PicHandler},
    port::{Port, PortManager},
    serial_println,
};

pub fn init(port_manager: &mut PortManager, interrupt_lookup: &InterruptLookup) {
    unsafe {
        let mut data = port_manager.request_port(0x60).unwrap();
        let mut status_and_command_register = port_manager.request_port(0x64).unwrap();
        init_ps2(&mut status_and_command_register, &mut data);

        interrupt_lookup.register_handler(InterruptHandler::Pic(PicHandler::new(
            IrqId::Pic1(1),
            move || {
                while status_and_command_register.read() & 2 > 0 {}
                let result = data.read();
                // serial_println!("{:#x}", result);
            },
        )));
    }
}

// https://wiki.osdev.org/%228042%22_PS/2_Controller#Initialising_the_PS/2_Controller
unsafe fn init_ps2(status_and_command_register: &mut Port, data: &mut Port) {
    // Disable devices
    status_and_command_register.write(0xAD);
    status_and_command_register.write(0xA7);

    // Flush output buf
    let _ = data.read();

    // Enable the first ps2 port
    status_and_command_register.write(0x20);
    let config = data.read();
    let new_config = config & !1 & !(1 << 6) & !(1 << 4);
    status_and_command_register.write(0x60);
    data.write(new_config);

    // Confirm the new config
    status_and_command_register.write(0x20);
    let confirm_config = data.read();
    assert_eq!(new_config, confirm_config);

    // Self test
    status_and_command_register.write(0xAA);
    let result = data.read();
    assert_eq!(0x55, result);

    // Restore config
    status_and_command_register.write(0x60);
    data.write(new_config);

    // Confirm the config
    status_and_command_register.write(0x20);
    let confirm_config = data.read();
    assert_eq!(new_config, confirm_config);

    // Skipping step 7 (Determine If There Are 2 Channels)

    // More tests
    status_and_command_register.write(0xAB);
    let result = data.read();
    assert_eq!(0, result);

    // Enable devices
    status_and_command_register.write(0xAE);

    // Enable interrupts
    status_and_command_register.write(0x20);
    let config = data.read();
    let new_config = config | 1;
    status_and_command_register.write(0x60);
    data.write(new_config);

    // Reset devices
    data.write(0xFF);

    // Detecting device in port 1

    // Disable scanning
    data.write(0xF5);
    while status_and_command_register.read() & 2 > 0 {}
    let result = data.read();
    assert_eq!(0xFA, result);

    // Identity command
    data.write(0xF2);
    while status_and_command_register.read() & 2 > 0 {}
    let result = data.read();
    assert_eq!(0xFA, result);

    while status_and_command_register.read() & 2 > 0 {}
    let first_id_byte = data.read();

    // TODO: timeout for second identity byte
    while status_and_command_register.read() & 2 > 0 {}
    let second_id_byte = data.read();

    // Re-enable scanning
    data.write(0xF4);
    while status_and_command_register.read() & 2 > 0 {}
    let result = data.read();
    assert_eq!(0xFA, result);

    match (first_id_byte, second_id_byte) {
        (0xAB, 0x83) => {
            info!("MF2 Keyboard ... [\x1b[32mConnected\x1b[00m]");
        }
        _ => {
            info!("Unknown ... [\x1b[32mConnected\x1b[00m]")
        }
    }
}
