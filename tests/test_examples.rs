use std::path::Path;

use lang_c::ast::*;

use kecc::run_ir::*;
use kecc::*;

fn test_dir<F>(path: &Path, f: F)
where
    F: Fn(&TranslationUnit),
{
    let mut parse = Parse::default();
    let dir = path.read_dir().expect("read_dir call failed");
    for entry in dir {
        let entry = ok_or!(entry, continue);
        let path = entry.path();
        if path.is_dir() {
            continue;
        }

        println!("[testing {:?}]", path);
        let test_unit = parse.translate(&path.as_path()).expect(
            &format!("parse failed {:?}", path.into_os_string().to_str().unwrap()).to_owned(),
        );
        f(&test_unit);
    }
}

#[test]
fn test_examples_write_c() {
    test_dir(Path::new("examples/"), write_c_test);
    test_dir(Path::new("examples/hw1"), write_c_test);
}

#[test]
fn test_examples_irgen() {
    test_dir(Path::new("examples/"), |test_unit| {
        let ir = Irgen::default()
            .translate(test_unit)
            .expect("failed to generate ir");

        // TODO: insert randomly generated command line arguments
        let args = Vec::new();

        assert_eq!(run_ir(&ir, args), Ok(Value::Int(1)));
    });
}
