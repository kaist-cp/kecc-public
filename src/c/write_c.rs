use std::io::{Result, Write};

use lang_c::ast::*;
use lang_c::span::Node;

use crate::write_base::*;

impl<T: WriteLine> WriteLine for Node<T> {
    fn write_line(&self, indent: usize, write: &mut dyn Write) -> Result<()> {
        self.node.write_line(indent, write)
    }
}

impl<T: WriteString> WriteString for Node<T> {
    fn write_string(&self) -> String {
        self.node.write_string()
    }
}

impl WriteLine for TranslationUnit {
    fn write_line(&self, indent: usize, write: &mut dyn Write) -> Result<()> {
        todo!()
    }
}

impl WriteString for Initializer {
    fn write_string(&self) -> String {
        todo!()
    }
}
