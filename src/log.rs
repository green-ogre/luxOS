use crate::{circular_buffer::CircularBuffer, port::PortManager, serial::SerialPort};
use core::{cell::UnsafeCell, fmt::Write};

pub static LOGGER: LogCell = LogCell(UnsafeCell::new(None));
pub struct LogCell(pub UnsafeCell<Option<Logger>>);
unsafe impl Sync for LogCell {}
impl LogCell {
    pub fn get(&self) -> Option<&Logger> {
        unsafe { (*self.0.get()).as_ref() }
    }
}

pub fn init(log_level: LogLevel, port_manager: &mut PortManager) {
    unsafe { (*LOGGER.0.get()) = Some(Logger::new(log_level, port_manager)) };
}

pub struct Logger {
    log_level: LogLevel,
    serial_port: SerialPort,
    buf: CircularBuffer<u8>,
}

impl Logger {
    const BUF_LEN: usize = 512;

    pub unsafe fn new(log_level: LogLevel, port_manager: &mut PortManager) -> Self {
        let buf = CircularBuffer::new(Self::BUF_LEN);
        let serial_port = SerialPort::new(port_manager);
        unsafe { serial_port.init() };

        Self {
            log_level,
            serial_port,
            buf,
        }
    }

    pub fn log(
        &self,
        logger: &'static Logger,
        log_level: LogLevel,
        args: core::fmt::Arguments<'_>,
    ) {
        if log_level.should_log(&self.log_level) {
            LogWriter { logger }.write_fmt(args).unwrap();
            logger.flush();
        }
    }

    pub fn flush(&self) {
        // while let Ok(byte) = self.rec.try_recv() {
        //     self.serial_port.write_byte(byte);
        // }

        while let Some(byte) = self.buf.read() {
            self.serial_port.write_byte(byte);
        }
    }
}

#[repr(u8)]
#[derive(Default, Clone, Copy, PartialEq, Eq)]
pub enum LogLevel {
    #[default]
    Debug,
    Info,
    Warning,
    Error,
}

impl LogLevel {
    pub fn should_log(&self, other: &Self) -> bool {
        match self {
            Self::Debug => *other == Self::Debug,
            Self::Info => *other == Self::Debug || *other == Self::Info,
            Self::Warning => {
                *other == Self::Debug || *other == Self::Info || *other == Self::Warning
            }
            Self::Error => {
                *other == Self::Debug
                    || *other == Self::Info
                    || *other == Self::Warning
                    || *other == Self::Error
            }
        }
    }
}

pub struct LogWriter {
    pub logger: &'static Logger,
}

impl Write for LogWriter {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        assert!(
            Logger::BUF_LEN > s.len(),
            "log is longer than buf len {}: {}",
            Logger::BUF_LEN,
            s
        );

        for c in s.bytes() {
            self.logger.buf.write(c);
        }
        Ok(())
    }
}

#[macro_export]
macro_rules! _fmt_log_level {
    ($lvl:expr, $color:expr) => {
        concat!(
            "[",
            module_path!(),
            ":",
            line!(),
            " ",
            concat!("\x1b[", $color, "m"),
            $lvl,
            "\x1b[00m",
            "] ",
        )
    };
}

#[macro_export]
macro_rules! debug {
    ($fmt:expr) => {
        $crate::log!(
            $crate::log::LogLevel::Debug,
            concat!($crate::_fmt_log_level!("DEBUG", 35), "{}\n"),
            $fmt
        )
    };
    ($fmt:expr, $($arg:tt)*) => {
        $crate::log!(
            $crate::log::LogLevel::Debug,
            concat!($crate::_fmt_log_level!("DEBUG", 35), $fmt, "\n"),
            $($arg)*
        )
    };
}

#[macro_export]
macro_rules! info {
    ($fmt:expr) => {
        $crate::log!(
            $crate::log::LogLevel::Info,
            concat!($crate::_fmt_log_level!("INFO", 32), "{}\n"),
            $fmt
        )
    };
    ($fmt:expr, $($arg:tt)*) => {
        $crate::log!(
            $crate::log::LogLevel::Info,
            concat!($crate::_fmt_log_level!("INFO", 32), $fmt, "\n"),
            $($arg)*
        )
    };
}

#[macro_export]
macro_rules! warn {
    ($fmt:expr) => {
        $crate::log!(
            $crate::log::LogLevel::Warning,
            concat!($crate::_fmt_log_level!("WARN", 33), "{}\n"),
            $fmt
        )
    };
    ($fmt:expr, $($arg:tt)*) => {
        $crate::log!(
            $crate::log::LogLevel::Warning,
            concat!($crate::_fmt_log_level!("WARN", 33), $fmt, "\n"),
            $($arg)*
        )
    };
}

#[macro_export]
macro_rules! error {
    ($fmt:expr) => {
        $crate::log!(
            $crate::log::LogLevel::Error,
            concat!($crate::_fmt_log_level!("ERROR", 31), "{}\n"),
            $fmt
        )
    };
    ($fmt:expr, $($arg:tt)*) => {
        $crate::log!(
            $crate::log::LogLevel::Error,
            concat!($crate::_fmt_log_level!("ERROR", 31), $fmt, "\n"),
            $($arg)*
        )
    };
}

#[macro_export]
macro_rules! log {
    ($log_lvl:expr, $($arg:tt)*) => {
        #[cfg(feature = "log")]
        {
            if let Some(logger) = $crate::log::LOGGER.get() {
                logger.log(logger, $log_lvl, format_args!($($arg)*));
            }
        }
    };
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {{
        if let Some(logger) = $crate::log::LOGGER.get() {
            use core::fmt::Write;
            $crate::log::LogWriter { logger }.write_fmt(format_args!($($arg)*)).unwrap();
            logger.flush();
        }
    }};
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!(b"\n"));
    ($($arg:tt)*) => {
        {
            $crate::print!($($arg)*);
            $crate::print!("\n");
        }
    };
}
