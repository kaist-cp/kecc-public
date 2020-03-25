use crate::asm::Asm;
use crate::ir;
use crate::Translate;

#[derive(Default)]
pub struct Asmgen {}

impl Translate<ir::TranslationUnit> for Asmgen {
    type Target = Asm;
    type Error = ();

    fn translate(&mut self, _source: &ir::TranslationUnit) -> Result<Self::Target, Self::Error> {
        todo!()
    }
}
