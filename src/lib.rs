pub mod algorithm;
pub mod base;
pub mod block;
pub mod cli;
pub mod counter;
pub mod interpreter;
pub mod machine;
pub mod range;
pub mod tape;
pub mod transition;

/// Print diagnostics only when enabled, without evaluating disabled arguments.
#[macro_export]
macro_rules! debug_println {
    ($enabled:expr, $($arg:tt)*) => {
        if $enabled { println!($($arg)*); }
    };
}
