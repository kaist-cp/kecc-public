use std::ffi::OsStr;
use std::path::Path;

use lang_c::ast::*;

use kecc::*;

fn test_dir<F>(path: &Path, ext: &OsStr, f: F)
where
    F: Fn(&TranslationUnit, &Path),
{
    let mut parse = Parse::default();
    let dir = path.read_dir().expect("read_dir call failed");
    for entry in dir {
        let entry = ok_or!(entry, continue);
        let path = entry.path();

        if !(path.is_file() && path.extension() == Some(ext)) {
            continue;
        }

        println!("[testing {:?}]", path);
        let test_unit = parse.translate(&path.as_path()).expect(
            &format!(
                "parse failed {:?}",
                path.clone().into_os_string().to_str().unwrap()
            )
            .to_owned(),
        );
        f(&test_unit, &path);
    }
}

#[test]
fn test_examples_write_c() {
    test_dir(Path::new("examples/c"), &OsStr::new("c"), test_write_c);
}

#[test]
fn test_examples_irgen() {
    test_dir(Path::new("examples/c"), &OsStr::new("c"), test_irgen);
}

// TODO: make it work!
#[test]
#[ignore]
fn test_examples_irparse() {
    test_dir(Path::new("examples/c"), &OsStr::new("c"), test_irparse);
}

#[test]
fn test_examples_simplify_cfg() {
    test_opt(
        &Path::new("examples/simplify_cfg/const_prop.input.ir"),
        &Path::new("examples/simplify_cfg/const_prop.output.ir"),
        &mut FunctionPass::<SimplifyCfgConstProp>::default(),
    );

    test_opt(
        &Path::new("examples/simplify_cfg/reach.input.ir"),
        &Path::new("examples/simplify_cfg/reach.output.ir"),
        &mut FunctionPass::<SimplifyCfgReach>::default(),
    );

    test_opt(
        &Path::new("examples/simplify_cfg/merge.input.ir"),
        &Path::new("examples/simplify_cfg/merge.output.ir"),
        &mut FunctionPass::<SimplifyCfgMerge>::default(),
    );

    test_opt(
        &Path::new("examples/simplify_cfg/empty.input.ir"),
        &Path::new("examples/simplify_cfg/empty.output.ir"),
        &mut FunctionPass::<SimplifyCfgEmpty>::default(),
    );
}

#[test]
fn test_examples_mem2reg() {
    test_opt(
        &Path::new("examples/mem2reg/mem2reg.input.ir"),
        &Path::new("examples/mem2reg/mem2reg.output.ir"),
        &mut Mem2reg::default(),
    );
}

#[test]
fn test_examples_gvn() {
    test_opt(
        &Path::new("examples/gvn/gvn.input.ir"),
        &Path::new("examples/gvn/gvn.output.ir"),
        &mut Gvn::default(),
    );
}

#[test]
fn test_examples_deadcode() {
    test_opt(
        &Path::new("examples/deadcode/deadcode.input.ir"),
        &Path::new("examples/deadcode/deadcode.output.ir"),
        &mut Deadcode::default(),
    );
}
