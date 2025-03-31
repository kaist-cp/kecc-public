use std::ffi::OsStr;
use std::path::Path;

use kecc::*;

use dir_test::{dir_test, Fixture};

fn test_dir<F>(path: &Path, ext: &OsStr, f: F)
where
    F: Fn(&Path),
{
    let dir = path.read_dir().expect("read_dir call failed");
    for entry in dir.filter_map(Result::ok) {
        let path = entry.path();

        if !(path.is_file() && path.extension() == Some(ext)) {
            continue;
        }

        f(&path);
    }
}

fn test_opt_between_dirs<O: Optimize<ir::TranslationUnit>>(from: &Path, to: &Path, opt: &mut O) {
    let from_dir = from.read_dir().expect("read_dir call failed");

    for entry in from_dir.filter_map(Result::ok) {
        let from_file_path = entry.path();

        if !(from_file_path.is_file() && from_file_path.extension() == Some(OsStr::new("ir"))) {
            continue;
        }

        let file_name = from_file_path
            .strip_prefix(from)
            .expect("`from_file_path` must have file name");
        let to_file_path = to.join(file_name);

        assert!(from_file_path.is_file());
        assert!(to_file_path.exists());
        assert!(to_file_path.is_file());

        println!("[testing {from_file_path:?} to {to_file_path:?}]");
        test_opt(&from_file_path, &to_file_path, opt);
    }
}

const HELLO_MAIN: &str = "hello_main";

const IRGEN_SMALL_TEST_IGNORE_LIST: [&str; 12] = [
    "examples/c/array.c",
    "examples/c/array2.c",
    "examples/c/array3.c",
    "examples/c/array4.c",
    "examples/c/array5.c",
    "examples/c/float.c",
    "examples/c/sizeof2.c",
    "examples/c/struct.c",
    "examples/c/struct2.c",
    "examples/c/struct3.c",
    "examples/c/struct4.c",
    "examples/c/temp2.c",
];

// TODO: Enable this test next semester.
const IRGEN_FULL_TEST_IGNORE_LIST: [&str; 1] = ["examples/c/side-effect.c"];

const ASMGEN_TEST_DIR_LIST: [&str; 5] = [
    "examples/ir0",
    "examples/ir1",
    "examples/ir2",
    "examples/ir3",
    "examples/ir4",
];

const ASMGEN_SMALL_TEST_IGNORE_LIST: [&str; 12] = [
    "array.ir",
    "array2.ir",
    "array3.ir",
    "array4.ir",
    "array5.ir",
    "float.ir",
    "sizeof2.ir",
    "struct.ir",
    "struct2.ir",
    "struct3.ir",
    "struct4.ir",
    "temp2.ir",
];

#[test]
fn test_examples_write_c() {
    println!("[testing write_c for \"examples/c/{HELLO_MAIN}.c\"]");
    test_write_c(Path::new(&format!("examples/c/{HELLO_MAIN}.c")));
    test_dir(Path::new("examples/c"), OsStr::new("c"), |path| {
        if !path.to_str().unwrap().contains(HELLO_MAIN) {
            println!("[testing write_c for {path:?}]");
            test_write_c(path);
        }
    });
}

#[dir_test(
    dir: "$CARGO_MANIFEST_DIR/regression_tests/c",
    glob: "*.c",
)]
fn test_regression_irgen(fixture: Fixture<&str>) {
    let path_str = fixture.path();
    println!("[testing irgen for '{path_str:?}']"); // TODO: Colors
    test_irgen(Path::new(path_str));
}

#[test]
fn test_examples_irgen_small() {
    println!("[testing irgen for \"examples/c/{HELLO_MAIN}.c\"]");
    test_irgen(Path::new(&format!("examples/c/{HELLO_MAIN}.c")));
    test_dir(Path::new("examples/c"), OsStr::new("c"), |path| {
        let path_str = &path.to_str().expect("`path` must be transformed to `&str`");
        if !IRGEN_SMALL_TEST_IGNORE_LIST.contains(path_str)
            && !IRGEN_FULL_TEST_IGNORE_LIST.contains(path_str)
            && !path_str.contains(HELLO_MAIN)
        {
            println!("[testing irgen for {path:?}]");
            test_irgen(path);
        }
    });
}

#[dir_test(
    dir: "$CARGO_MANIFEST_DIR/examples/c",
    glob: "*.c",
)]
fn test_examples_irgen(fixture: Fixture<&str>) {
    let path_str = fixture.path();
    println!("[testing irgen for '{path_str:?}']"); // TODO: Colors
    test_irgen(Path::new(path_str));
}
#[test]
fn test_examples_irgen_large() {
    test_dir(Path::new("examples/c"), OsStr::new("c"), |path| {
        let path_str = &path.to_str().expect("`path` must be transformed to `&str`");
    });
}

#[test]
fn test_examples_irparse() {
    test_dir(Path::new("examples/c"), OsStr::new("c"), test_irparse);
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

    test_opt_between_dirs(
        Path::new("examples/ir0"),
        Path::new("examples/ir1"),
        &mut SimplifyCfg::default(),
    );
}

#[test]
fn test_examples_mem2reg() {
    test_opt(
        &Path::new("examples/mem2reg/mem2reg.input.ir"),
        &Path::new("examples/mem2reg/mem2reg.output.ir"),
        &mut Mem2reg::default(),
    );

    test_opt_between_dirs(
        Path::new("examples/ir1"),
        Path::new("examples/ir2"),
        &mut Mem2reg::default(),
    );
}

#[test]
fn test_examples_deadcode() {
    test_opt(
        &Path::new("examples/deadcode/deadcode.input.ir"),
        &Path::new("examples/deadcode/deadcode.output.ir"),
        &mut Deadcode::default(),
    );

    test_opt_between_dirs(
        Path::new("examples/ir2"),
        Path::new("examples/ir3"),
        &mut Deadcode::default(),
    );
}

#[test]
fn test_examples_gvn() {
    test_opt(
        &Path::new("examples/gvn/gvn.input.ir"),
        &Path::new("examples/gvn/gvn.output.ir"),
        &mut Gvn::default(),
    );

    test_opt_between_dirs(
        Path::new("examples/ir3"),
        Path::new("examples/ir4"),
        &mut Gvn::default(),
    );
}

#[test]
fn test_examples_optimize() {
    test_opt_between_dirs(
        Path::new("examples/ir0"),
        Path::new("examples/opt"),
        &mut O1::default(),
    )
}

#[test]
fn test_examples_asmgen_small() {
    for dir in ASMGEN_TEST_DIR_LIST.iter() {
        test_dir(Path::new(dir), OsStr::new("ir"), |path| {
            if path.to_str().unwrap().contains(HELLO_MAIN) {
                println!("[testing asmgen for {path:?}]");
                test_asmgen(path);
            }
        });
    }
    for dir in ASMGEN_TEST_DIR_LIST.iter() {
        test_dir(Path::new(dir), OsStr::new("ir"), |path| {
            let file_name = &path
                .file_name()
                .expect("`path` must have a file name")
                .to_str()
                .expect("must be transformable to `&str`");
            if !ASMGEN_SMALL_TEST_IGNORE_LIST.contains(file_name) && !file_name.contains(HELLO_MAIN)
            {
                println!("[testing asmgen for {path:?}]");
                test_asmgen(path);
            }
        });
    }
}

#[test]
fn test_examples_asmgen_large() {
    for dir in ASMGEN_TEST_DIR_LIST.iter() {
        test_dir(Path::new(dir), OsStr::new("ir"), |path| {
            let file_name = &path
                .file_name()
                .expect("`path` must have a file name")
                .to_str()
                .expect("must be transformable to `&str`");
            if ASMGEN_SMALL_TEST_IGNORE_LIST.contains(file_name) && !file_name.contains(HELLO_MAIN)
            {
                println!("[testing asmgen for {path:?}]");
                test_asmgen(path);
            }
        });
    }
}

#[test]
fn test_examples_end_to_end() {
    test_dir(Path::new("examples/c"), OsStr::new("c"), |path| {
        println!("[testing end-to-end for {path:?}]");
        test_end_to_end(path);
    });
}
