use crate::ir;
use crate::*;

#[derive(Default)]
pub struct Mem2reg {}

impl Optimize<ir::TranslationUnit> for Mem2reg {
    fn optimize(&mut self, _code: &mut ir::TranslationUnit) -> bool {
        todo!()
    }
}
