mod gvn;
mod mem2reg;
mod simplify_cfg;

pub use gvn::Gvn;
pub use mem2reg::Mem2reg;
pub use simplify_cfg::SimplifyCfg;

use crate::ir;

pub trait Translate<S> {
    type Target;
    type Error;

    fn translate(&mut self, source: &S) -> Result<Self::Target, Self::Error>;
}

pub trait Optimize<T> {
    fn optimize(&mut self, code: &mut T) -> bool;
}

#[derive(Default)]
pub struct Repeat<O> {
    inner: O,
}

#[derive(Default)]
pub struct O0 {}

pub type O1 = Repeat<(SimplifyCfg, (Mem2reg, Gvn))>;

impl Optimize<ir::TranslationUnit> for O0 {
    fn optimize(&mut self, _code: &mut ir::TranslationUnit) -> bool {
        false
    }
}

impl<T, O1: Optimize<T>, O2: Optimize<T>> Optimize<T> for (O1, O2) {
    fn optimize(&mut self, code: &mut T) -> bool {
        let changed1 = self.0.optimize(code);
        let changed2 = self.1.optimize(code);
        changed1 || changed2
    }
}

impl<T, O: Optimize<T>> Optimize<T> for Repeat<O> {
    fn optimize(&mut self, code: &mut T) -> bool {
        if !self.inner.optimize(code) {
            return false;
        }

        while self.inner.optimize(code) {}
        true
    }
}
