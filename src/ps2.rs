use crate::{
    circular_buffer::CircularBuffer,
    info,
    interrupt::{InterruptHandler, InterruptLookup, IrqId, PicHandler},
    pic::Pic,
    port::{Port, PortManager},
};
use alloc::sync::Arc;

pub struct Ps2Keyboard {
    input: Arc<CircularBuffer<KeyboardInput>>,
}

impl Ps2Keyboard {
    pub fn new(
        port_manager: &mut PortManager,
        interrupt_lookup: &InterruptLookup,
        pic: &mut Pic,
    ) -> Self {
        let mut data = unsafe {
            port_manager
                .request_port(0x60)
                .expect("only one Ps2Keyboard driver may be active")
        };
        let mut status_and_command_register = unsafe {
            port_manager
                .request_port(0x64)
                .expect("only one Ps2Keyboard driver may be active")
        };
        unsafe { init_ps2(&mut status_and_command_register, &mut data) };

        let input = Arc::new(CircularBuffer::new(8));
        let hanlder_input = input.clone();

        let mut last_scan_code = 0;
        let pic_id = IrqId::Pic1(1);
        pic.unmask(pic_id);
        interrupt_lookup.register_handler(InterruptHandler::Pic(PicHandler::new(
            pic_id,
            move || {
                while unsafe { status_and_command_register.read() & 2 > 0 } {}
                let data = unsafe { data.read() };

                if data == 0xF0 {
                    last_scan_code = 0xF0;
                    return;
                }

                let state = if last_scan_code == 0xF0 {
                    KeyState::Released
                } else {
                    KeyState::Pressed
                };

                let key_code: KeyCode = ScanCode(data).into();
                last_scan_code = data;
                hanlder_input.write(KeyboardInput { key_code, state });
            },
        )));

        Self { input }
    }

    pub fn read_input_with(&self, mut f: impl FnMut(KeyboardInput)) {
        while let Some(input) = self.input.read() {
            f(input);
        }
    }
}

#[derive(Debug)]
pub struct KeyboardInput {
    pub key_code: KeyCode,
    pub state: KeyState,
}

#[derive(Debug, PartialEq, Eq)]
pub enum KeyCode {
    KeyA,

    KeyW,
    KeyS,
    KeyD,

    Unknown,
}

#[derive(Debug, PartialEq, Eq)]
pub enum KeyState {
    Pressed,
    Released,
}

#[derive(Debug)]
struct ScanCode(u8);

// https://wiki.osdev.org/PS/2_Keyboard
impl From<ScanCode> for KeyCode {
    fn from(value: ScanCode) -> Self {
        match value.0 {
            0x1C => KeyCode::KeyA,

            0x1D => KeyCode::KeyW,
            0x1B => KeyCode::KeyS,
            0x23 => KeyCode::KeyD,

            v => {
                crate::warn!("Unknown scane code: {:?}", v);
                KeyCode::Unknown
            }
        }
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
