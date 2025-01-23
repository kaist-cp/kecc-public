use core::ops::Deref;
use std::collections::HashMap;

use itertools::izip;
use lang_c::ast;

use crate::ir::*;
use crate::opt::opt_utils::*;
use crate::opt::*;

pub type Gvn = FunctionPass<GvnInner>;

#[derive(Default, Clone, Copy, Debug)]
pub struct GvnInner {}

impl Optimize<FunctionDefinition> for GvnInner {
    fn optimize(&mut self, code: &mut FunctionDefinition) -> bool {
        todo!()
    }
}
