#![deny(warnings)]
// Neccessary for skeleton code.
#![allow(unreachable_code)]
// Necessary to allow `iter.fold(false, |l, r| l || r)`. It's used when iteration should not be
// short-circuited.
#![allow(clippy::unnecessary_fold)]

mod tests;
mod utils;
mod write_base;

mod asm;
mod c;
pub mod ir;

mod asmgen;
mod irgen;
mod opt;

pub use tests::*;
pub use utils::*;
pub use write_base::write;

pub use c::Parse;
pub use ir::Parse as IrParse;

pub use asmgen::Asmgen;
pub use irgen::Irgen;
pub use opt::{
    Deadcode, FunctionPass, Gvn, Mem2reg, Optimize, Repeat, SimplifyCfg, SimplifyCfgConstProp,
    SimplifyCfgEmpty, SimplifyCfgMerge, SimplifyCfgReach, O0, O1,
};
