use crate::asm::Asm;
use crate::ir;
use crate::Translate;

#[derive(Default)]
pub struct Codegen {}

impl Translate<ir::TranslationUnit> for Codegen {
    type Target = Asm;
    type Error = ();

    fn translate(&mut self, _source: &ir::TranslationUnit) -> Result<Self::Target, Self::Error> {
        unimplemented!()
    }
}
