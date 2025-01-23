use core::ops::Deref;
use std::collections::{HashMap, HashSet};

use crate::ir::*;
use crate::opt::opt_utils::*;
use crate::opt::*;

pub type Deadcode = FunctionPass<Repeat<DeadcodeInner>>;

#[derive(Default, Clone, Copy, Debug)]
pub struct DeadcodeInner {}

impl Optimize<FunctionDefinition> for DeadcodeInner {
    fn optimize(&mut self, code: &mut FunctionDefinition) -> bool {
        todo!()
    }
}
