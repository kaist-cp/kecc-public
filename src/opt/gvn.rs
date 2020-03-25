use crate::ir;
use crate::*;

#[derive(Default)]
pub struct Gvn {}

impl Optimize<ir::TranslationUnit> for Gvn {
    fn optimize(&mut self, _code: &mut ir::TranslationUnit) -> bool {
        todo!()
    }
}
