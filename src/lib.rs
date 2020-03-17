#![deny(warnings)]

mod utils;

pub mod asm;
pub mod ir;

mod codegen;
mod irgen;
mod optimize;
mod parse;

pub mod run_ir;
mod write_asm;
mod write_base;
mod write_c;
mod write_ir;

pub mod assert_ast_equiv;
pub mod write_c_test;

pub use utils::*;

pub use asm::Asm;

pub use codegen::Codegen;
pub use irgen::Irgen;
pub use optimize::{O0, O1};
pub use parse::Parse;
pub use utils::{Optimize, Repeat, Translate};

pub use write_asm::write_asm;
pub use write_c::write_c;
pub use write_ir::write_ir;

pub use assert_ast_equiv::assert_ast_equiv;
pub use write_c_test::write_c_test;
