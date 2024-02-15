pub mod commands;
pub mod config;
pub mod err_msg_parse;
pub mod errlog;
pub mod gh;
pub mod issue;
pub mod util;

/// Module containing macros related to protocol words.
pub mod macros {
    #[macro_export]
    // These macros are needed because the normal ones panic when there's a broken pipe.
    // This is especially problematic for CLI tools that are frequently piped into `head` or `grep -q`
    macro_rules! pipe_println {
        () => (print!("\n"));
        ($fmt:expr) => ({
            writeln!(std::io::stdout(), $fmt)
        });
        ($fmt:expr, $($arg:tt)*) => ({
            writeln!(std::io::stdout(), $fmt, $($arg)*)
        })
    }
    pub use pipe_println;

    #[macro_export]
    macro_rules! pipe_print {
        () => (print!("\n"));
        ($fmt:expr) => ({
            write!(std::io::stdout(), $fmt)
        });
        ($fmt:expr, $($arg:tt)*) => ({
            write!(std::io::stdout(), $fmt, $($arg)*)
        })
    }

    pub use pipe_print;
}
