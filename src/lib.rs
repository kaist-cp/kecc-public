#![deny(warnings)]
#![allow(unreachable_code)]

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

pub use asmgen::Asmgen;
pub use irgen::Irgen;
pub use opt::{Deadcode, Gvn, Mem2reg, Optimize, Repeat, SimplifyCfg, O0, O1};
