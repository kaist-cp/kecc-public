use std::io::{Result, Write};

use crate::asm::*;
use crate::write_base::*;

const INDENT: usize = 4;

impl WriteLine for Asm {
    fn write_line(&self, indent: usize, write: &mut dyn Write) -> Result<()> {
        self.unit.write_line(indent, write)
    }
}

impl WriteLine for TranslationUnit {
    fn write_line(&self, indent: usize, write: &mut dyn Write) -> Result<()> {
        for function in &self.functions {
            function.write_line(indent, write)?;
        }

        for variable in &self.variables {
            variable.write_line(indent, write)?;
        }

        Ok(())
    }
}

impl<T: WriteLine> WriteLine for Section<T> {
    fn write_line(&self, indent: usize, write: &mut dyn Write) -> Result<()> {
        for directive in &self.header {
            write_indent(indent + INDENT, write)?;
            writeln!(write, "{directive}")?;
        }
        self.body.write_line(indent, write)?;

        Ok(())
    }
}

impl WriteLine for Function {
    fn write_line(&self, indent: usize, write: &mut dyn Write) -> Result<()> {
        for block in &self.blocks {
            block.write_line(indent, write)?;
        }

        Ok(())
    }
}

impl WriteLine for Variable {
    fn write_line(&self, indent: usize, write: &mut dyn Write) -> Result<()> {
        writeln!(write, "{}:", self.label.0)?;
        for directive in &self.directives {
            write_indent(indent + INDENT, write)?;
            writeln!(write, "{directive}")?;
        }

        Ok(())
    }
}

impl WriteLine for Block {
    fn write_line(&self, indent: usize, write: &mut dyn Write) -> Result<()> {
        if let Some(label) = &self.label {
            writeln!(write, "{label}:")?;
        }

        for instruction in &self.instructions {
            write_indent(indent + INDENT, write)?;
            writeln!(write, "{instruction}")?;
        }

        Ok(())
    }
}
