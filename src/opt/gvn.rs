use crate::opt::FunctionPass;
use crate::*;

pub type Gvn = FunctionPass<GvnInner>;

#[derive(Default, Clone, Copy, Debug)]
pub struct GvnInner {}

impl Optimize<ir::FunctionDefinition> for GvnInner {
    fn optimize(&mut self, _code: &mut ir::FunctionDefinition) -> bool {
        todo!("homework 5")
    }
}
