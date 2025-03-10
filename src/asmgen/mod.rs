use std::collections::{BTreeMap, HashMap, HashSet, VecDeque};

use lang_c::ast;

use crate::ir::HasDtype;
use crate::opt::opt_utils;
use crate::{Translate, asm, ir};

#[derive(Debug)]
pub struct Asmgen {}

impl Default for Asmgen {
    fn default() -> Self {
        todo!()
    }
}

impl Translate<ir::TranslationUnit> for Asmgen {
    type Target = asm::Asm;
    type Error = ();

    fn translate(&mut self, source: &ir::TranslationUnit) -> Result<Self::Target, Self::Error> {
        todo!()
    }
}
