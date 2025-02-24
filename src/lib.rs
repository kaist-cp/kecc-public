//! KECC: KAIST Educational C Compiler.

#![deny(clippy::all)]
// #![deny(rustdoc::all)]
#![deny(warnings)]
// Tries to deny all rustc allow lints.
// <https://doc.rust-lang.org/rustc/lints/listing/allowed-by-default.html>
#![deny(
    absolute_paths_not_starting_with_crate,
    elided_lifetimes_in_paths,
    explicit_outlives_requirements,
    keyword_idents,
    let_underscore_drop,
    macro_use_extern_crate,
    meta_variable_misuse,
    missing_abi,
    // These are stupid.
    // missing_copy_implementations,
    // missing_debug_implementations,
    // TODO
    // missing_docs,
    non_ascii_idents,
    noop_method_call,
    rust_2021_incompatible_closure_captures,
    rust_2021_incompatible_or_patterns,
    rust_2021_prefixes_incompatible_syntax,
    rust_2021_prelude_collisions,
    single_use_lifetimes,
    trivial_casts,
    trivial_numeric_casts,
    unreachable_pub,
    unsafe_code,
    unstable_features,
    // Necessary for `build-bin` trick.
    // unused_crate_dependencies,
    unused_extern_crates,
    unused_import_braces,
    unused_lifetimes,
    unused_macro_rules,
    unused_qualifications,
    unused_results,
    // This is stupid. Allowed for more flexible variants.
    // variant_size_differences,
)]
// For skeleton code.
#![allow(unused)]

mod tests;
mod utils;
mod write_base;

pub mod asm;
mod c;
pub mod ir;

mod asmgen;
mod irgen;
mod opt;

pub use asmgen::Asmgen;
pub use c::Parse;
pub use ir::{Parse as IrParse, Visualizer as IrVisualizer};
pub use irgen::Irgen;
pub use opt::{
    Deadcode, FunctionPass, Gvn, Mem2reg, O0, O1, Optimize, Repeat, SimplifyCfg,
    SimplifyCfgConstProp, SimplifyCfgEmpty, SimplifyCfgMerge, SimplifyCfgReach,
};
pub use tests::*;
pub use utils::*;
pub use write_base::write;
