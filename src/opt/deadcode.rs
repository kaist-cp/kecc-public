use crate::ir::*;
use crate::opt::FunctionPass;
use crate::*;

pub type Deadcode = FunctionPass<Repeat<DeadcodeInner>>;

#[derive(Default, Clone, Copy, Debug)]
pub struct DeadcodeInner {}

impl Optimize<FunctionDefinition> for DeadcodeInner {
    fn optimize(&mut self, _code: &mut FunctionDefinition) -> bool {
        todo!("homework 6")
    }
}
