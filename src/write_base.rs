use std::io::{Result, Write};

/// Write `indent` number of double spaces to `write`.
pub fn write_indent(indent: usize, write: &mut dyn Write) -> Result<()> {
    write!(write, "{}", "  ".repeat(indent))
}

/// A trait for writing a type to a `Write` stream with a new line.
pub trait WriteLine {
    /// Write `self` to `write`, starting at `indent` number of double spaces, with a newline at the
    /// end.
    fn write_line(&self, indent: usize, write: &mut dyn Write) -> Result<()>;
}

/// Essentially the same as [`ToString`].
///
/// Exists to make some foreign types into a string.
pub trait WriteString {
    /// See [`ToString::to_string`].
    fn write_string(&self) -> String;
}

impl<T: WriteString> WriteString for Box<T> {
    fn write_string(&self) -> String {
        (**self).write_string()
    }
}

impl<T: WriteString> WriteString for &T {
    fn write_string(&self) -> String {
        (*self).write_string()
    }
}

// Might be useful for debugging.
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
