use crate::*;

mod deadcode;
mod gvn;
mod mem2reg;
pub(crate) mod opt_utils;
mod simplify_cfg;

pub use deadcode::Deadcode;
pub use gvn::Gvn;
pub use mem2reg::Mem2reg;
pub use simplify_cfg::{
    SimplifyCfg, SimplifyCfgConstProp, SimplifyCfgEmpty, SimplifyCfgMerge, SimplifyCfgReach,
};

use crate::ir;

pub trait Optimize<T> {
    fn optimize(&mut self, code: &mut T) -> bool;
}

pub type O0 = Null;
pub type O1 = Repeat<(SimplifyCfg, (Mem2reg, (Gvn, Deadcode)))>;

#[derive(Default, Clone, Copy, Debug)]
pub struct Null;

#[derive(Default, Debug)]
pub struct Repeat<O> {
    inner: O,
}

#[derive(Default, Debug)]
pub struct FunctionPass<T: Optimize<ir::FunctionDefinition>> {
    inner: T,
}

impl Optimize<ir::TranslationUnit> for Null {
    fn optimize(&mut self, _code: &mut ir::TranslationUnit) -> bool {
        false
    }
}

impl<T, O1: Optimize<T>, O2: Optimize<T>> Optimize<T> for (O1, O2) {
    fn optimize(&mut self, code: &mut T) -> bool {
        self.0.optimize(code) | self.1.optimize(code)
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

impl<T> Optimize<ir::TranslationUnit> for FunctionPass<T>
where
    T: Optimize<ir::FunctionDefinition>,
{
    fn optimize(&mut self, code: &mut ir::TranslationUnit) -> bool {
        code.decls
            .values_mut()
            .fold(false, |b, decl| b | self.optimize(decl))
    }
}

impl<T> Optimize<ir::Declaration> for FunctionPass<T>
where
    T: Optimize<ir::FunctionDefinition>,
{
    fn optimize(&mut self, code: &mut ir::Declaration) -> bool {
        let Some((_, Some(fdef))) = code.get_function_mut() else {
            return false;
        };
        self.inner.optimize(fdef)
    }
}
