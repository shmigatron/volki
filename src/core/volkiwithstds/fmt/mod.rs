//! Formatting utilities — re-exports core::fmt + custom macros.

// Re-export core::fmt types
pub use core::fmt::{Debug, Display, Formatter, Result, Write as FmtWrite};

/// Create a Vec (replaces `vec!`).
///
/// Usage: `vvec![1, 2, 3]`
#[macro_export]
macro_rules! vvec {
    () => { $crate::core::volkiwithstds::collections::Vec::new() };
    ($($x:expr),+ $(,)?) => {{
        let mut v = $crate::core::volkiwithstds::collections::Vec::new();
        $(v.push($x);)+
        v
    }};
}

/// Create a String from a literal (replaces `String::from(...)`).
///
/// Usage: `vstr!("hello")`
#[macro_export]
macro_rules! vstr {
    ($s:expr) => { $crate::core::volkiwithstds::collections::String::from($s) };
}

/// Box a value and coerce it to a trait object.
///
/// Usage: `vbox!(value => dyn Trait)`
#[macro_export]
macro_rules! vbox {
    ($val:expr => $dyn_ty:ty) => {{
        let raw = $crate::core::volkiwithstds::collections::Box::into_raw(
            $crate::core::volkiwithstds::collections::Box::new($val),
        ) as *mut $dyn_ty;
        unsafe { $crate::core::volkiwithstds::collections::Box::from_raw(raw) }
    }};
}

/// Format into our custom String type (replaces `format!`).
///
/// Usage: `vformat!("hello {}", name)`
#[macro_export]
macro_rules! vformat {
    ($($arg:tt)*) => {{
        let mut s = $crate::core::volkiwithstds::collections::String::new();
        core::fmt::write(&mut s, format_args!($($arg)*)).unwrap();
        s
    }};
}

/// Print to stderr (replaces `eprint!`).
///
/// Usage: `veprint!("error: {}", msg)`
#[macro_export]
macro_rules! veprint {
    ($($arg:tt)*) => {{
        use $crate::core::volkiwithstds::io::traits::Write;
        let mut stderr = $crate::core::volkiwithstds::io::stderr();
        let _ = stderr.write_fmt(format_args!($($arg)*));
    }};
}

/// Print to stderr with newline (replaces `eprintln!`).
///
/// Usage: `veprintln!("error: {}", msg)`
#[macro_export]
macro_rules! veprintln {
    () => {{
        use $crate::core::volkiwithstds::io::traits::Write;
        let mut stderr = $crate::core::volkiwithstds::io::stderr();
        let _ = stderr.write_all(b"\n");
    }};
    ($($arg:tt)*) => {{
        use $crate::core::volkiwithstds::io::traits::Write;
        let mut stderr = $crate::core::volkiwithstds::io::stderr();
        let _ = stderr.write_fmt(format_args!($($arg)*));
        let _ = stderr.write_all(b"\n");
    }};
}

/// Print to stdout (replaces `print!`).
///
/// Usage: `vprint!("hello {}", name)`
#[macro_export]
macro_rules! vprint {
    ($($arg:tt)*) => {{
        use $crate::core::volkiwithstds::io::traits::Write;
        let mut stdout = $crate::core::volkiwithstds::io::stdout();
        let _ = stdout.write_fmt(format_args!($($arg)*));
    }};
}

/// Print to stdout with newline (replaces `println!`).
///
/// Usage: `vprintln!("hello {}", name)`
#[macro_export]
macro_rules! vprintln {
    () => {{
        use $crate::core::volkiwithstds::io::traits::Write;
        let mut stdout = $crate::core::volkiwithstds::io::stdout();
        let _ = stdout.write_all(b"\n");
    }};
    ($($arg:tt)*) => {{
        use $crate::core::volkiwithstds::io::traits::Write;
        let mut stdout = $crate::core::volkiwithstds::io::stdout();
        let _ = stdout.write_fmt(format_args!($($arg)*));
        let _ = stdout.write_all(b"\n");
    }};
}
