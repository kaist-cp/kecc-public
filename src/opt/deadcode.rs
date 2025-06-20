use core::ops::Deref;

use crate::ir::*;
use crate::opt::opt_utils::*;
use crate::opt::*;
use rustc_hash::{FxHashMap, FxHashSet};

pub type Deadcode = FunctionPass<Repeat<DeadcodeInner>>;

#[derive(Default, Clone, Copy, Debug)]
pub struct DeadcodeInner {}

impl Optimize<FunctionDefinition> for DeadcodeInner {
    fn optimize(&mut self, code: &mut FunctionDefinition) -> bool {
        todo!()
    }
}
