use crate::ir::*;
use crate::*;

pub type SimplifyCfg = Repeat<(SimplifyCfgConstProp, (SimplifyCfgReach, SimplifyCfgMerge))>;

impl Optimize<TranslationUnit> for SimplifyCfg {
    fn optimize(&mut self, code: &mut TranslationUnit) -> bool {
        code.decls.iter_mut().any(|(_, decl)| self.optimize(decl))
    }
}

impl Optimize<Declaration> for SimplifyCfg {
    fn optimize(&mut self, code: &mut Declaration) -> bool {
        let (_fsig, fdef) = some_or!(code.get_function_mut(), return false);
        let fdef = some_or!(fdef, return false);
        self.optimize(fdef)
    }
}

/// Simplifies block exits by propagating constants.
#[derive(Default)]
pub struct SimplifyCfgConstProp {}

/// Retains only those blocks that are reachable from the init.
#[derive(Default)]
pub struct SimplifyCfgReach {}

/// Merges two blocks if a block is pointed to only by another
#[derive(Default)]
pub struct SimplifyCfgMerge {}

impl Optimize<FunctionDefinition> for SimplifyCfgConstProp {
    fn optimize(&mut self, _code: &mut FunctionDefinition) -> bool {
        todo!("homework 3")
    }
}

impl Optimize<FunctionDefinition> for SimplifyCfgReach {
    fn optimize(&mut self, _code: &mut FunctionDefinition) -> bool {
        todo!("homework 3")
    }
}

impl Optimize<FunctionDefinition> for SimplifyCfgMerge {
    fn optimize(&mut self, _code: &mut FunctionDefinition) -> bool {
        todo!("homework 3")
    }
}
