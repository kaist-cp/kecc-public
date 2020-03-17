use std::io::{Result, Write};

#[inline]
pub fn write_indent(indent: usize, write: &mut dyn Write) -> Result<()> {
    write!(write, "{}", "  ".repeat(indent))
}

pub trait WriteLine {
    fn write_line(&self, indent: usize, write: &mut dyn Write) -> Result<()>;
}

pub trait WriteString {
    fn write_string(&self) -> String;
}

pub trait WriteOp {
    fn write_operation(&self) -> String;
}
