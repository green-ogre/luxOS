use crate::{
    interrupt::{
        InterruptFlag, InterruptFrame, InterruptHandler, InterruptLookup, IrqId, PicHandler,
    },
    interrupt_guard,
    port::{Port, PortManager},
    serial_println,
};

pub struct Cmos {
    pub register_select: Port,
    pub data: Port,
}

impl Cmos {
    pub fn new(port_manager: &mut PortManager) -> Self {
        let register_select = unsafe { port_manager.request_port(0x70).unwrap() };
        let data = unsafe { port_manager.request_port(0x71).unwrap() };

        Self {
            register_select,
            data,
        }
    }

    /// Lacks inner-second precision. May sleep for `0 < 1` more or less than expected.
    pub fn sleep(&self, mut secs: usize) {
        let mut last_second = self.second();
        loop {
            if secs == 0 {
                break;
            }

            let second = self.second();
            if last_second != second {
                secs -= 1;
                last_second = second;
            }
        }
    }

    pub fn second(&self) -> u8 {
        self.query_rtc_reg(0x00)
    }

    pub fn minute(&self) -> u8 {
        self.query_rtc_reg(0x02)
    }

    pub fn hour(&self) -> u8 {
        self.query_rtc_reg(0x04)
    }

    fn query_rtc_reg(&self, reg: u8) -> u8 {
        while self.update_in_progress() {
            // serial_println!("Cmos update in progress, spinning...");
        }
        let mut val = self.read_register(reg);

        loop {
            while self.update_in_progress() {
                // serial_println!("Cmos update in progress, spinning...");
            }

            let last_val = self.read_register(reg);
            if val != last_val {
                val = last_val;
            } else {
                break;
            }
        }

        let reg_b = self.read_register(0x0B);
        if (reg_b & 0x04) == 0 && reg != 0x04 {
            val = (val & 0x0F) + ((val / 16) * 10);
        } else if (reg_b & 0x04 == 0) && reg == 0x04 {
            val = ((val & 0x0F) + (((val & 0x70) / 16) * 10)) | (val & 0x80);
        }

        val
    }

    pub fn get_rtc(&self) -> Rtc {
        while self.update_in_progress() {
            serial_println!("Cmos update in progress, spinning...");
        }
        let mut second = self.read_register(0x00);
        let mut minute = self.read_register(0x02);
        let mut hour = self.read_register(0x04);
        let mut day = self.read_register(0x07);
        let mut month = self.read_register(0x08);
        let mut year = self.read_register(0x09);

        // NOTE: In practice, this never actually necessary, but it is a much more robust solution to
        // prevent one time bugs.
        //
        // let mut loops = 0;
        loop {
            while self.update_in_progress() {
                serial_println!("Cmos update in progress, spinning...");
            }
            let last_second = self.read_register(0x00);
            let last_minute = self.read_register(0x02);
            let last_hour = self.read_register(0x04);
            let last_day = self.read_register(0x07);
            let last_month = self.read_register(0x08);
            let last_year = self.read_register(0x09);

            if last_second != second
                || last_minute != minute
                || last_hour != hour
                || last_day != day
                || last_month != month
                || last_year != year
            {
                second = last_second;
                minute = last_minute;
                hour = last_hour;
                day = last_day;
                month = last_month;
                year = last_year;
                // loops += 1;
            } else {
                // serial_println!("rtc fetch took {} confirm read", loops + 1);
                break;
            }
        }

        let reg_b = self.read_register(0x0B);
        let bcd_to_dec = |bcd: &mut u8| *bcd = (*bcd & 0x0F) + ((*bcd / 16) * 10);
        if (reg_b & 0x04) == 0 {
            bcd_to_dec(&mut second);
            bcd_to_dec(&mut minute);
            hour = ((hour & 0x0F) + (((hour & 0x70) / 16) * 10)) | (hour & 0x80);
            bcd_to_dec(&mut day);
            bcd_to_dec(&mut month);
            bcd_to_dec(&mut year);
        }

        Rtc {
            second,
            minute,
            hour,
            day,
            month,
            year,
        }
    }

    pub fn read_register(&self, register: u8) -> u8 {
        self.select_register(register);
        unsafe { self.data.read() }
    }

    fn select_register(&self, register: u8) {
        let nmi_disable_bit = 0;
        unsafe {
            self.register_select
                .write((nmi_disable_bit << 7) | register);
        }
    }

    fn update_in_progress(&self) -> bool {
        unsafe {
            self.register_select.write(0x0A);
            self.data.read() & 0x80 > 0
        }
    }
}

#[derive(Debug)]
pub struct Rtc {
    pub second: u8,
    pub minute: u8,
    pub hour: u8,
    pub day: u8,
    pub month: u8,
    pub year: u8,
}

impl Rtc {
    pub fn enable_irq(
        port_manager: &mut PortManager,
        interrupt_flag: &mut InterruptFlag,
        interrupt_lookup: &InterruptLookup,
    ) {
        let cmos = Cmos::new(port_manager);

        interrupt_guard!(interrupt_flag, {
            unsafe {
                // disable NMI
                cmos.register_select.write(0x8B);
                let prev = cmos.data.read();
                cmos.register_select.write(0x8B);
                cmos.data.write(prev | 0x40);

                // enable NMI & flush
                cmos.read_register(0xC);
            }

            interrupt_lookup.register_handler(InterruptHandler::Pic(PicHandler::new(
                IrqId::Pic2(0),
                move || {
                    // Flush c register, allow next interrupt
                    //
                    // https://wiki.osdev.org/RTC
                    cmos.read_register(0xC);
                },
            )));
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{debug, test_case};

    test_case!(time, {
        let mut port_manager = PortManager::default();
        let cmos = Cmos::new(&mut port_manager);
        let rtc = cmos.get_rtc();
        debug!("{:?}", rtc);
        test_assert_eq!(cmos.second(), rtc.second);
        test_assert_eq!(cmos.minute(), rtc.minute);
        test_assert_eq!(cmos.hour(), rtc.hour);
    });
}
