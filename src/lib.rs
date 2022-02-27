//! KECC: KAIST Educational C Compiler.

// Tries to deny all lints (`rustc -W help`).
#![deny(warnings)]
#![deny(absolute_paths_not_starting_with_crate)]
#![deny(anonymous_parameters)]
// #![deny(box_pointers)]
#![deny(deprecated_in_future)]
#![deny(elided_lifetimes_in_paths)]
#![deny(explicit_outlives_requirements)]
#![deny(rustdoc::invalid_html_tags)]
#![deny(keyword_idents)]
#![deny(macro_use_extern_crate)]
#![deny(missing_debug_implementations)]
// #![deny(missing_docs)] TODO
#![deny(rustdoc::missing_doc_code_examples)]
#![deny(non_ascii_idents)]
#![deny(pointer_structural_match)]
// #![deny(single_use_lifetimes)]
#![deny(trivial_numeric_casts)]
#![deny(unaligned_references)]
// #![deny(unreachable_pub)]
#![deny(unstable_features)]
// Necessary for `build-bin` trick.
// #![deny(unused_crate_dependencies)]
#![deny(unused_extern_crates)]
#![deny(unused_import_braces)]
#![deny(unused_lifetimes)]
#![deny(unused_qualifications)]
#![deny(unused_results)]
// #![deny(variant_size_differences)]
#![deny(rust_2018_idioms)]
#![deny(rustdoc::all)]
// Necessary for skeleton code.
#![allow(unreachable_code)]
// Necessary to allow `iter.fold(false, |l, r| l || r)`. It's used when iteration should not be
// short-circuited.
#![allow(clippy::unnecessary_fold)]
#![allow(clippy::result_unit_err)]
#![allow(clippy::vec_init_then_push)]
#![allow(clippy::collapsible_match)]

mod tests;
mod utils;
mod write_base;

pub mod asm;
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
pub use ir::Visualizer as IrVisualizer;

pub use asmgen::Asmgen;
pub use irgen::Irgen;
pub use opt::{
    Deadcode, FunctionPass, Gvn, Mem2reg, Optimize, Repeat, SimplifyCfg, SimplifyCfgConstProp,
    SimplifyCfgEmpty, SimplifyCfgMerge, SimplifyCfgReach, O0, O1,
};
