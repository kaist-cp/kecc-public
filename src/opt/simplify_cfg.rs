use std::collections::{HashMap, HashSet};
use std::ops::Deref;

use itertools::izip;

use crate::ir::*;
use crate::opt::opt_utils::*;
use crate::opt::*;

pub type SimplifyCfg = FunctionPass<
    Repeat<(
        SimplifyCfgConstProp,
        (SimplifyCfgReach, (SimplifyCfgMerge, SimplifyCfgEmpty)),
    )>,
>;

/// Simplifies block exits by propagating constants.
#[derive(Default, Clone, Copy, Debug)]
pub struct SimplifyCfgConstProp {}

/// Retains only those blocks that are reachable from the init.
#[derive(Default, Clone, Copy, Debug)]
pub struct SimplifyCfgReach {}

/// Merges two blocks if a block is pointed to only by another
#[derive(Default, Clone, Copy, Debug)]
pub struct SimplifyCfgMerge {}

/// Removes empty blocks
#[derive(Default, Clone, Copy, Debug)]
pub struct SimplifyCfgEmpty {}

impl Optimize<FunctionDefinition> for SimplifyCfgConstProp {
    fn optimize(&mut self, code: &mut FunctionDefinition) -> bool {
        todo!()
    }
}

impl Optimize<FunctionDefinition> for SimplifyCfgReach {
    fn optimize(&mut self, code: &mut FunctionDefinition) -> bool {
        todo!()
    }
}

impl Optimize<FunctionDefinition> for SimplifyCfgMerge {
    fn optimize(&mut self, code: &mut FunctionDefinition) -> bool {
        todo!()
    }
}

impl Optimize<FunctionDefinition> for SimplifyCfgEmpty {
    fn optimize(&mut self, code: &mut FunctionDefinition) -> bool {
        todo!()
    }
}
