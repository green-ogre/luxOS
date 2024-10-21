use crate::lock::spinlock::SpinLock;

pub fn init(log_level: LogLevel) {
    LOGGER.lock().log_level = Some(log_level);
}

#[derive(Default)]
pub struct Logger {
    pub log_level: Option<LogLevel>,
}

impl Logger {
    pub const fn new() -> Self {
        Self { log_level: None }
    }
}

pub static LOGGER: SpinLock<Logger> = SpinLock::new(Logger::new());

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
        #[cfg(feature = "log")]
        $crate::_log!(
            $crate::log::LogLevel::Debug,
            concat!($crate::_fmt_log_level!("DEBUG", 35), "{}\n"),
            $fmt
        )
    };
    ($fmt:expr, $($arg:tt)*) => {
        #[cfg(feature = "log")]
        $crate::_log!(
            $crate::log::LogLevel::Debug,
            concat!($crate::_fmt_log_level!("DEBUG", 35), $fmt, "\n"),
            $($arg)*
        )
    };
}

#[macro_export]
macro_rules! info {
    ($fmt:expr) => {
        #[cfg(feature = "log")]
        $crate::_log!(
            $crate::log::LogLevel::Info,
            concat!($crate::_fmt_log_level!("INFO", 32), "{}\n"),
            $fmt
        )
    };
    ($fmt:expr, $($arg:tt)*) => {
        #[cfg(feature = "log")]
        $crate::_log!(
            $crate::log::LogLevel::Info,
            concat!($crate::_fmt_log_level!("INFO", 32), $fmt, "\n"),
            $($arg)*
        )
    };
}

#[macro_export]
macro_rules! warn {
    ($fmt:expr) => {
        #[cfg(feature = "log")]
        $crate::_log!(
            $crate::log::LogLevel::Warning,
            concat!($crate::_fmt_log_level!("WARN", 33), "{}\n"),
            $fmt
        )
    };
    ($fmt:expr, $($arg:tt)*) => {
        #[cfg(feature = "log")]
        $crate::_log!(
            $crate::log::LogLevel::Warning,
            concat!($crate::_fmt_log_level!("WARN", 33), $fmt, "\n"),
            $($arg)*
        )
    };
}

#[macro_export]
macro_rules! error {
    ($fmt:expr) => {
        #[cfg(feature = "log")]
        $crate::_log!(
            $crate::log::LogLevel::Error,
            concat!($crate::_fmt_log_level!("ERROR", 31), "{}\n"),
            $fmt
        )
    };
    ($fmt:expr, $($arg:tt)*) => {
        #[cfg(feature = "log")]
        $crate::_log!(
            $crate::log::LogLevel::Error,
            concat!($crate::_fmt_log_level!("ERROR", 31), $fmt, "\n"),
            $($arg)*
        )
    };
}

#[macro_export]
macro_rules! _log {
    ($log_lvl:expr, $($arg:tt)*) => {
        {
            if let Some(logger_lvl) = $crate::log::LOGGER.lock().log_level {
                if $log_lvl.should_log(&logger_lvl) {
                    $crate::serial::_print(format_args!($($arg)*))
                }
            }
        }
    };
}
