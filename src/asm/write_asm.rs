use std::io::{Result, Write};

use crate::asm::Asm;
use crate::write_base::WriteLine;

impl WriteLine for Asm {
    fn write_line(&self, _indent: usize, _write: &mut dyn Write) -> Result<()> {
        todo!()
    }
}
