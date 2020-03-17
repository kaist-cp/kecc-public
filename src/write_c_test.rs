use lang_c::ast::*;
use std::fs::File;
use tempfile::tempdir;

use crate::*;

pub fn write_c_test(unit: &TranslationUnit) {
    let temp_dir = tempdir().expect("temp dir creation failed");
    let temp_file_path = temp_dir.path().join("temp.c");
    let mut temp_file = File::create(&temp_file_path).unwrap();

    write_c(&unit, &mut temp_file).unwrap();

    let new_unit = Parse::default()
        .translate(&temp_file_path.as_path())
        .expect("parse failed while parsing file from implemented printer");
    drop(temp_file);
    assert_ast_equiv(&unit, &new_unit);
    temp_dir.close().expect("temp dir deletion failed");
}
