use core::ops::Deref;

use itertools::izip;
use lang_c::ast;
use rustc_hash::FxHashMap;

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
