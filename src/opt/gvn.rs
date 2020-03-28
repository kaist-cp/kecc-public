use crate::opt::FunctionPass;
use crate::*;

pub type Gvn = FunctionPass<Repeat<GvnInner>>;

#[derive(Default)]
pub struct GvnInner {}

impl Optimize<ir::FunctionDefinition> for GvnInner {
    fn optimize(&mut self, _code: &mut ir::FunctionDefinition) -> bool {
        todo!("homework 5")
    }
}
