use std::fs::{self, File};
use std::io::{stderr, Read, Write};
use std::path::Path;
use std::process::{Command, Stdio};
use tempfile::tempdir;
use wait_timeout::ChildExt;

use crate::*;

// Rust sets an exit code of 101 when the process panicked.
// So, we decide KECC sets an exit code of 102 after 101 when the test skipped.
pub const SKIP_TEST: i32 = 102;

pub fn test_write_c(path: &Path) {
    // Check if the file has .c extension
    assert_eq!(path.extension(), Some(std::ffi::OsStr::new("c")));
    let unit = c::Parse::default()
        .translate(&path)
        .unwrap_or_else(|_| panic!("parse failed {}", path.display()));

    let temp_dir = tempdir().expect("temp dir creation failed");
    let temp_file_path = temp_dir.path().join("temp.c");
    let mut temp_file = File::create(&temp_file_path).unwrap();

    crate::write(&unit, &mut temp_file).unwrap();

    let new_unit = c::Parse::default()
        .translate(&temp_file_path.as_path())
        .expect("parse failed while parsing the output from implemented printer");
    drop(temp_file);
    c::assert_ast_equiv(&unit, &new_unit);
    temp_dir.close().expect("temp dir deletion failed");
}

pub fn test_irgen(path: &Path) {
    // Check if the file has .c extension
    assert_eq!(path.extension(), Some(std::ffi::OsStr::new("c")));
    let unit = c::Parse::default()
        .translate(&path)
        .unwrap_or_else(|_| panic!("parse failed {}", path.display()));

    // Test parse
    c::Parse::default()
        .translate(&path)
        .expect("failed to parse the given program");

    let file_path = path.display().to_string();
    let bin_path = path.with_extension("irgen").as_path().display().to_string();

    // Compile c file: If fails, test is vacuously success
    if !Command::new("gcc")
        .args(&[
            "-fsanitize=undefined",
            "-fno-sanitize-recover=all",
            "-O1",
            &file_path,
            "-o",
            &bin_path,
        ])
        .stderr(Stdio::null())
        .status()
        .unwrap()
        .success()
    {
        ::std::process::exit(SKIP_TEST);
    }

    // Execute compiled executable
    let mut child = Command::new(fs::canonicalize(bin_path.clone()).unwrap())
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to execute the compiled executable");

    Command::new("rm")
        .arg(bin_path)
        .status()
        .expect("failed to remove compiled executable");

    let status = some_or!(
        child
            .wait_timeout_ms(500)
            .expect("failed to obtain exit status from child process"),
        {
            println!("timeout occurs");
            child.kill().unwrap();
            child.wait().unwrap();
            ::std::process::exit(SKIP_TEST);
        }
    );

    if child
        .stderr
        .expect("`stderr` of `child` must be `Some`")
        .bytes()
        .next()
        .is_some()
    {
        println!("error occurs");
        ::std::process::exit(SKIP_TEST);
    }

    let status = some_or_exit!(status.code(), SKIP_TEST);

    let ir = Irgen::default()
        .translate(&unit)
        .unwrap_or_else(|irgen_error| panic!("{}", irgen_error));
    let args = Vec::new();
    let result = ir::interp(&ir, args).unwrap_or_else(|interp_error| panic!("{}", interp_error));
    // We only allow main function whose return type is `int`
    let (value, width, is_signed) = result.get_int().expect("non-integer value occurs");
    assert_eq!(width, 32);
    assert!(is_signed);

    // When obtain status from `gcc` executable process, value is truncated to byte size.
    // For this reason, we make `fuzzer` generate the C source code which returns value
    // typecasted to `unsigned char`. However, during `creduce` reduce the code, typecasting
    // may be deleted. So, we truncate result value to byte size one more time here.
    println!("gcc: {}, kecc: {}", status as u8, value as u8);
    assert_eq!(status as u8, value as u8);
}

pub fn test_irparse(path: &Path) {
    // Check if the file has .c extension
    assert_eq!(path.extension(), Some(std::ffi::OsStr::new("c")));
    let unit = c::Parse::default()
        .translate(&path)
        .unwrap_or_else(|_| panic!("parse failed {}", path.display()));

    // Test parse
    c::Parse::default()
        .translate(&path)
        .expect("failed to parse the given program");

    let temp_dir = tempdir().expect("temp dir creation failed");

    // Test for original IR
    let ir = Irgen::default()
        .translate(&unit)
        .unwrap_or_else(|irgen_error| panic!("{}", irgen_error));
    let temp_file_path = temp_dir.path().join("ir0.ir");
    let mut temp_file = File::create(&temp_file_path).unwrap();
    crate::write(&ir, &mut temp_file).unwrap();

    let ir0 = ir::Parse::default()
        .translate(&temp_file_path.as_path())
        .expect("parse failed while parsing the output from implemented printer");
    drop(temp_file);
    assert_eq!(ir, ir0);

    // Test for optimized IR
    let ir1 =
        test_irparse_for_optimized_ir(ir0, &temp_dir.path().join("ir1.ir"), SimplifyCfg::default());
    let ir2 =
        test_irparse_for_optimized_ir(ir1, &temp_dir.path().join("ir2.ir"), Mem2reg::default());
    let ir3 =
        test_irparse_for_optimized_ir(ir2, &temp_dir.path().join("ir3.ir"), Deadcode::default());
    let _ = test_irparse_for_optimized_ir(ir3, &temp_dir.path().join("ir4.ir"), Gvn::default());

    temp_dir.close().expect("temp dir deletion failed");
}

#[inline]
fn test_irparse_for_optimized_ir<O: Optimize<ir::TranslationUnit>>(
    mut ir: ir::TranslationUnit,
    temp_file_path: &Path,
    mut opt: O,
) -> ir::TranslationUnit {
    opt.optimize(&mut ir);
    let mut temp_file = File::create(temp_file_path).unwrap();
    crate::write(&ir, &mut temp_file).unwrap();

    let optimized_ir = ir::Parse::default()
        .translate(&temp_file_path)
        .expect("parse failed while parsing the output from implemented printer");
    drop(temp_file);
    assert_eq!(ir, optimized_ir);

    optimized_ir
}

pub fn test_opt<P1: AsRef<Path>, P2: AsRef<Path>, O: Optimize<ir::TranslationUnit>>(
    from: &P1,
    to: &P2,
    opt: &mut O,
) {
    let from = ir::Parse::default()
        .translate(from)
        .expect("parse failed while parsing the output from implemented printer");
    let mut ir = from.clone();
    let to = ir::Parse::default()
        .translate(to)
        .expect("parse failed while parsing the output from implemented printer");
    opt.optimize(&mut ir);

    if !ir.is_equiv(&to) {
        stderr()
            .lock()
            .write_fmt(format_args!(
                "[test_opt] actual outcome mismatches with the expected outcome.\n\n[before opt]"
            ))
            .unwrap();
        crate::write(&from, &mut stderr()).unwrap();
        stderr()
            .lock()
            .write_fmt(format_args!("\n[after opt]"))
            .unwrap();
        crate::write(&ir, &mut stderr()).unwrap();
        stderr()
            .lock()
            .write_fmt(format_args!("\n[after opt (expected)]"))
            .unwrap();
        crate::write(&to, &mut stderr()).unwrap();
        panic!("[test_opt]");
    }
}

pub fn test_asmgen(path: &Path) {
    // TODO: delete ignore list in the future
    let ignore_list = vec![
        "examples/asmgen/struct.ir",
        "examples/asmgen/struct2.ir",
        "examples/asmgen/struct3.ir",
        "examples/asmgen/temp2.ir",
    ];
    if ignore_list.contains(&path.to_str().expect("`path` must be transformed to `&str`")) {
        println!("skip test");
        return;
    }

    // Check if the file has .ir extension
    assert_eq!(path.extension(), Some(std::ffi::OsStr::new("ir")));
    let unit = ir::Parse::default()
        .translate(&path)
        .unwrap_or_else(|_| panic!("parse failed {}", path.display()));

    // Execute IR
    let args = Vec::new();
    let result = ir::interp(&unit, args).unwrap_or_else(|interp_error| panic!("{}", interp_error));
    // We only allow main function whose return type is `int`
    let (value, width, is_signed) = result.get_int().expect("non-integer value occurs");
    assert_eq!(width, 32);
    assert!(is_signed);

    // Generate RISC-V assembly from IR
    let asm = Asmgen::default()
        .translate(&unit)
        .expect("fail to create riscv assembly code");
    let asm_path = path.with_extension("S").as_path().display().to_string();
    let mut buffer = File::create(Path::new(&asm_path)).expect("need to success creating file");
    write(&asm, &mut buffer).unwrap();

    // Link to an RISC-V executable
    let bin_path = path
        .with_extension("asmgen")
        .as_path()
        .display()
        .to_string();
    if !Command::new("riscv64-linux-gnu-gcc-10")
        .args(&["-static", &asm_path, "-o", &bin_path])
        .stderr(Stdio::null())
        .status()
        .unwrap()
        .success()
    {
        ::std::process::exit(SKIP_TEST);
    }

    Command::new("rm")
        .arg(asm_path)
        .status()
        .expect("failed to remove assembly code file");

    // Emulate the executable
    let mut child = Command::new("qemu-riscv64-static")
        .args(&[&bin_path])
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to execute the compiled executable");

    Command::new("rm")
        .arg(bin_path)
        .status()
        .expect("failed to remove compiled executable");

    let status = some_or!(
        child
            .wait_timeout_ms(500)
            .expect("failed to obtain exit status from child process"),
        {
            println!("timeout occurs");
            child.kill().unwrap();
            child.wait().unwrap();
            ::std::process::exit(SKIP_TEST);
        }
    );

    if child
        .stderr
        .expect("`stderr` of `child` must be `Some`")
        .bytes()
        .next()
        .is_some()
    {
        println!("error occurs");
        ::std::process::exit(SKIP_TEST);
    }

    let qemu_status = some_or_exit!(status.code(), SKIP_TEST);

    println!("kecc interp: {}, qemu: {}", value as u8, qemu_status as u8);
    assert_eq!(value as u8, qemu_status as u8);
}

// TODO: test all the way down to assembly
pub fn test_end_to_end(path: &Path) {
    // Check if the file has .c extension
    assert_eq!(path.extension(), Some(std::ffi::OsStr::new("c")));
    let unit = c::Parse::default()
        .translate(&path)
        .unwrap_or_else(|_| panic!("parse failed {}", path.display()));

    // Test parse
    c::Parse::default()
        .translate(&path)
        .expect("failed to parse the given program");

    let file_path = path.display().to_string();
    let bin_path = path
        .with_extension("endtoend")
        .as_path()
        .display()
        .to_string();

    // Compile c file: If fails, test is vacuously success
    if !Command::new("gcc")
        .args(&[
            "-fsanitize=undefined",
            "-fno-sanitize-recover=all",
            "-O1",
            &file_path,
            "-o",
            &bin_path,
        ])
        .stderr(Stdio::null())
        .status()
        .unwrap()
        .success()
    {
        ::std::process::exit(SKIP_TEST);
    }

    // Execute compiled executable
    let mut child = Command::new(fs::canonicalize(bin_path.clone()).unwrap())
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to execute the compiled executable");

    Command::new("rm")
        .arg(bin_path)
        .status()
        .expect("failed to remove compiled executable");

    let status = some_or!(
        child
            .wait_timeout_ms(500)
            .expect("failed to obtain exit status from child process"),
        {
            println!("timeout occurs");
            child.kill().unwrap();
            child.wait().unwrap();
            ::std::process::exit(SKIP_TEST);
        }
    );

    if child
        .stderr
        .expect("`stderr` of `child` must be `Some`")
        .bytes()
        .next()
        .is_some()
    {
        println!("error occurs");
        ::std::process::exit(SKIP_TEST);
    }

    let status = some_or_exit!(status.code(), SKIP_TEST);

    let mut ir = Irgen::default()
        .translate(&unit)
        .unwrap_or_else(|irgen_error| panic!("{}", irgen_error));
    O1::default().optimize(&mut ir);
    let args = Vec::new();
    let result = ir::interp(&ir, args).unwrap_or_else(|interp_error| panic!("{}", interp_error));
    // We only allow main function whose return type is `int`
    let (value, width, is_signed) = result.get_int().expect("non-integer value occurs");
    assert_eq!(width, 32);
    assert!(is_signed);

    // When obtain status from `gcc` executable process, value is truncated to byte size.
    // For this reason, we make `fuzzer` generate the C source code which returns value
    // typecasted to `unsigned char`. However, during `creduce` reduce the code, typecasting
    // may be deleted. So, we truncate result value to byte size one more time here.
    println!("gcc: {}, kecc: {}", status as u8, value as u8);
    assert_eq!(status as u8, value as u8);
}
