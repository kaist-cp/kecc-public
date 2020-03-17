use crate::ir;
use crate::{Optimize, Repeat};

#[derive(Default)]
pub struct O0 {}

#[derive(Default)]
pub struct Mem2reg {}

#[derive(Default)]
pub struct Gvn {}

pub type O1 = Repeat<(Mem2reg, Gvn)>;

impl Optimize<ir::TranslationUnit> for O0 {
    fn optimize(&mut self, _code: &mut ir::TranslationUnit) -> bool {
        false
    }
}

impl Optimize<ir::TranslationUnit> for Mem2reg {
    fn optimize(&mut self, _code: &mut ir::TranslationUnit) -> bool {
        unimplemented!()
    }
}

impl Optimize<ir::TranslationUnit> for Gvn {
    fn optimize(&mut self, _code: &mut ir::TranslationUnit) -> bool {
        unimplemented!()
    }
}
