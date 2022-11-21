//! KECC: KAIST Educational C Compiler.

#![deny(clippy::all)]
#![deny(rustdoc::all)]
#![deny(warnings)]
// Tries to deny all rustc allow lints.
// <https://doc.rust-lang.org/rustc/lints/listing/allowed-by-default.html>
#![deny(absolute_paths_not_starting_with_crate)]
// Old, historical lint
// #![deny(box_pointers)]
#![deny(elided_lifetimes_in_paths)]
#![deny(explicit_outlives_requirements)]
#![deny(keyword_idents)]
#![deny(let_underscore_drop)]
#![deny(macro_use_extern_crate)]
#![deny(meta_variable_misuse)]
#![deny(missing_abi)]
#![deny(missing_copy_implementations)]
#![deny(missing_debug_implementations)]
// TODO
// #![deny(missing_docs)]
#![deny(non_ascii_idents)]
#![deny(noop_method_call)]
#![deny(pointer_structural_match)]
#![deny(rust_2021_incompatible_closure_captures)]
#![deny(rust_2021_incompatible_or_patterns)]
#![deny(rust_2021_prefixes_incompatible_syntax)]
#![deny(rust_2021_prelude_collisions)]
// Necessary for skeleton code.
// #![deny(single_use_lifetimes)]
#![deny(trivial_casts)]
#![deny(trivial_numeric_casts)]
// Necessary for skeleton code.
// #![deny(unreachable_pub)]
#![deny(unsafe_code)]
#![deny(unsafe_op_in_unsafe_fn)]
#![deny(unstable_features)]
// Necessary for `build-bin` trick.
// #![deny(unused_crate_dependencies)]
#![deny(unused_extern_crates)]
#![deny(unused_import_braces)]
#![deny(unused_lifetimes)]
#![deny(unused_macro_rules)]
#![deny(unused_qualifications)]
#![deny(unused_results)]
#![deny(unused_tuple_struct_fields)]
// Allowed for more flexible variants.
// #![deny(variant_size_differences)]

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
