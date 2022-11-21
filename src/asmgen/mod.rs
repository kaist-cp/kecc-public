use crate::asm;
use crate::ir;
use crate::Translate;

#[derive(Default, Clone, Copy, Debug)]
pub struct Asmgen {}

impl Translate<ir::TranslationUnit> for Asmgen {
    type Target = asm::Asm;
    type Error = ();

    fn translate(&mut self, _source: &ir::TranslationUnit) -> Result<Self::Target, Self::Error> {
        todo!("homework 7")
    }
}
