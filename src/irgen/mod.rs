use core::fmt;

use lang_c::ast::*;

use crate::*;

#[derive(Default)]
pub struct Irgen {}

#[derive(Debug, PartialEq)]
pub struct IrgenError {}

impl fmt::Display for IrgenError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "IrgenError")
    }
}

impl Translate<TranslationUnit> for Irgen {
    type Target = ir::TranslationUnit;
    type Error = IrgenError;

    fn translate(&mut self, _unit: &TranslationUnit) -> Result<Self::Target, Self::Error> {
        todo!("homework 2")
    }
}
