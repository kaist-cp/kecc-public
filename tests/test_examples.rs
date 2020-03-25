use std::ffi::OsStr;
use std::path::Path;

use lang_c::ast::*;

use kecc::*;

fn test_dir<F>(path: &Path, f: F)
where
    F: Fn(&TranslationUnit, &Path),
{
    let mut parse = Parse::default();
    let dir = path.read_dir().expect("read_dir call failed");
    for entry in dir {
        let entry = ok_or!(entry, continue);
        let path = entry.path();

        if !(path.is_file() && path.extension() == Some(&OsStr::new("c"))) {
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
    test_dir(Path::new("examples/"), test_write_c);
    test_dir(Path::new("examples/hw1"), test_write_c);
}

#[test]
fn test_examples_irgen() {
    test_dir(Path::new("examples/"), test_irgen);
}
