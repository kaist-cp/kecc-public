use lang_c::ast::*;
use std::fs::{self, File};
use std::path::Path;
use std::process::{Command, Stdio};
use tempfile::tempdir;

use crate::*;

pub fn test_write_c(unit: &TranslationUnit, _path: &Path) {
    let temp_dir = tempdir().expect("temp dir creation failed");
    let temp_file_path = temp_dir.path().join("temp.c");
    let mut temp_file = File::create(&temp_file_path).unwrap();

    crate::write(unit, &mut temp_file).unwrap();

    let new_unit = c::Parse::default()
        .translate(&temp_file_path.as_path())
        .expect("parse failed while parsing the output from implemented printer");
    drop(temp_file);
    c::assert_ast_equiv(&unit, &new_unit);
    temp_dir.close().expect("temp dir deletion failed");
}

pub fn test_irgen(unit: &TranslationUnit, path: &Path) {
    // Check if the file has .c extension
    assert_eq!(path.extension(), Some(std::ffi::OsStr::new("c")));

    // Test parse
    c::Parse::default()
        .translate(&path)
        .expect("failed to parse the given program");

    let file_path = path.display().to_string();
    let bin_path = path.with_extension("exe").as_path().display().to_string();

    // Compile c file: If fails, test is vacuously success
    if !Command::new("gcc")
        .args(&["-O1", &file_path, "-o", &bin_path])
        .stderr(Stdio::null())
        .status()
        .unwrap()
        .success()
    {
        return;
    }

    // Execute compiled executable
    let status = Command::new(fs::canonicalize(bin_path.clone()).unwrap())
        .status()
        .expect("failed to execute the compiled executable")
        .code()
        .expect("failed to return an exit code");

    // Remove compiled executable
    Command::new("rm")
        .arg(bin_path)
        .status()
        .expect("failed to remove compiled executable");

    let ir = Irgen::default()
        .translate(unit)
        .expect("failed to generate ir");

    let args = Vec::new();
    assert_eq!(
        ir::interp(&ir, args),
        Ok(ir::Value::int(status as u128, 32, true))
    );
}
