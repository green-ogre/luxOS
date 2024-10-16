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
            concat!($crate::_fmt_log_level!("DEBUG", 35), "{}\n"),
            $fmt
        )
    };
    ($fmt:expr, $($arg:tt)*) => {
        #[cfg(feature = "log")]
        $crate::_log!(
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
            concat!($crate::_fmt_log_level!("INFO", 32), "{}\n"),
            $fmt
        )
    };
    ($fmt:expr, $($arg:tt)*) => {
        #[cfg(feature = "log")]
        $crate::_log!(
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
            concat!($crate::_fmt_log_level!("WARN", 33), "{}\n"),
            $fmt
        )
    };
    ($fmt:expr, $($arg:tt)*) => {
        #[cfg(feature = "log")]
        $crate::_log!(
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
            concat!($crate::_fmt_log_level!("ERROR", 31), "{}\n"),
            $fmt
        )
    };
    ($fmt:expr, $($arg:tt)*) => {
        #[cfg(feature = "log")]
        $crate::_log!(
            concat!($crate::_fmt_log_level!("ERROR", 31), $fmt, "\n"),
            $($arg)*
        )
    };
}

#[macro_export]
macro_rules! _log {
    ($($arg:tt)*) => {
        $crate::serial::_print(format_args!($($arg)*))
    };
}
