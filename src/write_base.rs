use std::io::{Result, Write};

/// TODO(document)
#[inline]
pub fn write_indent(indent: usize, write: &mut dyn Write) -> Result<()> {
    write!(write, "{}", "  ".repeat(indent))
}

/// TODO(document)
pub trait WriteLine {
    /// TODO(document)
    fn write_line(&self, indent: usize, write: &mut dyn Write) -> Result<()>;
}

/// TODO(document)
pub trait WriteString {
    /// TODO(document)
    fn write_string(&self) -> String;
}

/// TODO(document)
pub trait WriteOp {
    /// TODO(document)
    fn write_operation(&self) -> String;
}

/// TODO(document)
pub fn write<T: WriteLine>(t: &T, write: &mut dyn Write) -> Result<()> {
    t.write_line(0, write)
}
