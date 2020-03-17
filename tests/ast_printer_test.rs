use std::path::Path;

use kecc::*;

#[test]
fn ast_printer_test() {
    let mut parse = Parse::default();
    let dir_path = Path::new("examples/");
    let dir = dir_path.read_dir().expect("read_dir call failed");
    for entry in dir {
        let test_file = ok_or!(entry, continue);
        let test_unit = parse.translate(&test_file.path().as_path()).expect(
            &format!(
                "parse failed {:?}",
                test_file.path().into_os_string().to_str().unwrap()
            )
            .to_owned(),
        );
        write_c_test(&test_unit);
    }
}
