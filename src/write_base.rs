use std::io::{Result, Write};

/// Write `indent` number of double spaces to `write`.
#[inline]
pub fn write_indent(indent: usize, write: &mut dyn Write) -> Result<()> {
    write!(write, "{}", "  ".repeat(indent))
}

/// A trait for writing a type to a `Write` stream with a new line.
pub trait WriteLine {
    /// Write `self` to `write`, starting at `indent` number of double spaces, with a newline at the
    /// ned.
    fn write_line(&self, indent: usize, write: &mut dyn Write) -> Result<()>;
}

/// Format types to a String.
///
/// Most cases, `fmt::Display` is used to format a type to a string. However, in some cases, we
/// can't implement `fmt::Display` for a type as it is defined in another crate. In such cases, we
/// can implement this trait to format the type to a string.
pub trait WriteString {
    /// Change a type into a String.
    fn write_string(&self) -> String;
}

impl<T: WriteString> WriteString for Box<T> {
    fn write_string(&self) -> String {
        use core::ops::Deref;
        self.deref().write_string()
    }
}

impl<T: WriteString> WriteString for &T {
    fn write_string(&self) -> String {
        (*self).write_string()
    }
}

impl<T: WriteString> WriteString for Option<T> {
    fn write_string(&self) -> String {
        if let Some(this) = self {
            this.write_string()
        } else {
            "".to_string()
        }
    }
}

/// Write `t` to `write`.
pub fn write<T: WriteLine>(t: &T, write: &mut dyn Write) -> Result<()> {
    t.write_line(0, write)
}
