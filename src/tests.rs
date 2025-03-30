use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::Duration;

use lang_c::*;
use rand::Rng;
use tempfile::tempdir;
use wait_timeout::ChildExt;

use crate::write_base::WriteLine;
use crate::*;

const NONCE_NAME: &str = "nonce";

fn modify_c(path: &Path, rand_num: i32) -> String {
    let mut src = File::open(path).expect("`path` must exist");
    let mut data = String::new();
    let _ = src
        .read_to_string(&mut data)
        .expect("`src` must be converted to string");
    drop(src);

    let from = format!("int {NONCE_NAME} = 1");
    let to = format!("int {NONCE_NAME} = {rand_num}");
    data.replace(&from, &to)
}

fn ast_initializer(number: i32) -> ast::Initializer {
    let expr = ast::Expression::Constant(Box::new(span::Node::new(
        ast::Constant::Integer(ast::Integer {
            base: ast::IntegerBase::Decimal,
            number: Box::from(number.to_string()),
            suffix: ast::IntegerSuffix {
                size: ast::IntegerSize::Int,
                unsigned: false,
                imaginary: false,
            },
        }),
        span::Span::none(),
    )));

    ast::Initializer::Expression(Box::new(span::Node::new(expr, span::Span::none())))
}

fn modify_ir(unit: &mut ir::TranslationUnit, rand_num: i32) {
    for (name, decl) in &mut unit.decls {
        if name == NONCE_NAME {
            let (dtype, _) = decl.get_variable().expect("`decl` must be variable");
            let initializer = ast_initializer(rand_num);
            let new_decl = ir::Declaration::Variable {
                dtype: dtype.clone(),
                initializer: Some(initializer),
            };

            *decl = new_decl;
        }
    }
}

fn modify_asm(unit: &mut asm::Asm, rand_num: i32) {
    for variable in &mut unit.unit.variables {
        let body = &mut variable.body;
        if body.label == asm::Label(NONCE_NAME.to_string()) {
            let directive =
                asm::Directive::try_from_data_size(asm::DataSize::Word, rand_num as u64);

            body.directives = vec![directive];
        }
    }
}

// Rust sets an exit code of 101 when the process panicked.
// Set exit code of 102 after 101 to denote that the test skipped.
const SKIP_TEST: i32 = 102;

/// Tests write_c.
pub fn test_write_c(path: &Path) {
    assert_eq!(path.extension(), Some(std::ffi::OsStr::new("c")));
    let unit = Parse
        .translate(&path)
        .unwrap_or_else(|_| panic!("parse failed {}", path.display()));

    let temp_dir = tempdir().expect("temp dir creation failed");
    let temp_file_path = temp_dir.path().join("temp.c");
    let mut temp_file = File::create(&temp_file_path).unwrap();

    write(&unit, &mut temp_file).unwrap();

    {
        let mut buf = String::new();
        // FIXME: For some reason we cannot reuse `temp_file`.
        let _ = File::open(&temp_file_path)
            .unwrap()
            .read_to_string(&mut buf)
            .unwrap();

        println!("{}", buf);
    }

    let new_unit = Parse
        .translate(&temp_file_path.as_path())
        .expect("parse failed while parsing the output from implemented printer");

    if !unit.is_equiv(&new_unit) {
        let mut buf = String::new();
        // FIXME: For some reason we cannot reuse `temp_file`.
        let _ = File::open(&temp_file_path)
            .unwrap()
            .read_to_string(&mut buf)
            .unwrap();

        println!("{}", buf);

        panic!("[write-c] Failed to correctly write {path:?}.\n\n[incorrect result]\n\n{buf}");
    }
}

/// Tests irgen.
pub fn test_irgen(path: &Path) {
    // Check if the file has .c extension
    assert_eq!(path.extension(), Some(std::ffi::OsStr::new("c")));
    let unit = Parse
        .translate(&path)
        .unwrap_or_else(|_| panic!("parse failed {}", path.display()));

    let mut ir = Irgen::default()
        .translate(&unit)
        .unwrap_or_else(|irgen_error| panic!("{}", irgen_error));

    println!("IR: {ir:#?}");
    println!();
    ir.write_line(0, &mut io::stdout()).unwrap();

    let rand_num = rand::rng().random_range(1..100);
    let new_c = modify_c(path, rand_num);
    modify_ir(&mut ir, rand_num);

    // compile recolved c example
    let temp_dir = tempdir().expect("temp dir creation failed");
    let temp_file_path = temp_dir.path().join("temp.c");
    let mut temp_file = File::create(&temp_file_path).unwrap();
    temp_file.write_all(new_c.as_bytes()).unwrap();

    let file_path = temp_file_path.display().to_string();
    let bin_path = temp_file_path
        .with_extension("irgen")
        .as_path()
        .display()
        .to_string();

    // Compile c file: If fails, test is vacuously success
    if !Command::new("clang")
        .args([
            "-fsanitize=float-divide-by-zero",
            "-fsanitize=undefined",
            "-fno-sanitize-recover=all",
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
    let mut child = Command::new(fs::canonicalize(bin_path).unwrap())
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to execute the compiled executable");

    let Some(status) = child
        .wait_timeout(Duration::from_millis(1000))
        .expect("failed to obtain exit status from child process")
    else {
        println!("timeout occurs");
        child.kill().unwrap();
        let _ = child.wait().unwrap();
        ::std::process::exit(SKIP_TEST);
    };

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

    // Interpret resolved ir
    let args = Vec::new();
    let result = ir::interp(&ir, args).unwrap_or_else(|interp_error| panic!("{interp_error}"));
    // We only allow a main function whose return type is `int`
    let (value, width, is_signed) = result.get_int().expect("non-integer value occurs");
    assert_eq!(width, 32);
    assert!(is_signed);

    // When obtaining status from `clang` executable process, the status value is truncated to byte
    // size. For this reason, we make `fuzzer` generate the C source code which returns values
    // typecasted to `unsigned char`. However, during `creduce` to reduce the code, typecasting may
    // be nullified. So, we truncate the result value to byte size one more time here.
    println!("clang (expected): {}, kecc: {}", status as u8, value as u8);
    if status as u8 != value as u8 {
        let mut stderr = io::stderr().lock();
        stderr
            .write_fmt(format_args!(
                "[irgen] Failed to correctly generate {path:?}.\n\n [incorrect ir]"
            ))
            .unwrap();
        write(&ir, &mut stderr).unwrap();
        drop(stderr);
        panic!("[irgen]");
    }
}

/// Tests irparse.
pub fn test_irparse(path: &Path) {
    // Check if the file has .c extension
    assert_eq!(path.extension(), Some(std::ffi::OsStr::new("c")));
    let unit = Parse
        .translate(&path)
        .unwrap_or_else(|_| panic!("parse failed {}", path.display()));

    // Test parse
    let _unused = Parse
        .translate(&path)
        .expect("failed to parse the given program");

    let temp_dir = tempdir().expect("temp dir creation failed");

    // Test for original IR
    let ir = Irgen::default()
        .translate(&unit)
        .unwrap_or_else(|irgen_error| panic!("{}", irgen_error));
    let temp_file_path = temp_dir.path().join("ir0.ir");
    let mut temp_file = File::create(&temp_file_path).unwrap();
    write(&ir, &mut temp_file).unwrap();

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
    let _unused =
        test_irparse_for_optimized_ir(ir3, &temp_dir.path().join("ir4.ir"), Gvn::default());

    temp_dir.close().expect("temp dir deletion failed");
}

#[inline]
fn test_irparse_for_optimized_ir<O: Optimize<ir::TranslationUnit>>(
    mut ir: ir::TranslationUnit,
    temp_file_path: &Path,
    mut opt: O,
) -> ir::TranslationUnit {
    let _ = opt.optimize(&mut ir);
    let mut temp_file = File::create(temp_file_path).unwrap();
    write(&ir, &mut temp_file).unwrap();

    let optimized_ir = ir::Parse::default()
        .translate(&temp_file_path)
        .expect("parse failed while parsing the output from implemented printer");
    drop(temp_file);
    assert_eq!(ir, optimized_ir);

    optimized_ir
}

/// Tests optimizations.
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
    let _ = opt.optimize(&mut ir);

    if !ir.is_equiv(&to) {
        let mut stderr = io::stderr().lock();
        stderr
            .write_fmt(format_args!(
                "[test_opt] actual outcome mismatches with the expected outcome.\n\n[before opt]"
            ))
            .unwrap();
        write(&from, &mut stderr).unwrap();
        stderr.write_fmt(format_args!("\n[after opt]")).unwrap();
        write(&ir, &mut stderr).unwrap();
        stderr
            .write_fmt(format_args!("\n[after opt (expected)]"))
            .unwrap();
        write(&to, &mut stderr).unwrap();
        drop(stderr);
        panic!("[test_opt]");
    }
}

/// Tests asmgen.
pub fn test_asmgen(path: &Path) {
    // Check if the file has .ir extension
    assert_eq!(path.extension(), Some(std::ffi::OsStr::new("ir")));
    let mut ir = ir::Parse::default()
        .translate(&path)
        .unwrap_or_else(|_| panic!("parse failed {}", path.display()));

    // Generate RISC-V assembly from IR
    let mut asm = Asmgen::default()
        .translate(&ir)
        .expect("fail to create riscv assembly code");

    let rand_num = rand::rng().random_range(1..100);
    modify_ir(&mut ir, rand_num);
    modify_asm(&mut asm, rand_num);

    // Execute IR
    let args = Vec::new();
    let result = ir::interp(&ir, args).unwrap_or_else(|interp_error| panic!("{}", interp_error));
    // We only allow main function whose return type is `int`
    let (value, width, is_signed) = result.get_int().expect("non-integer value occurs");
    assert_eq!(width, 32);
    assert!(is_signed);

    let temp_dir = tempdir().expect("temp dir creation failed");
    let asm_path = temp_dir.path().join("temp.S");
    let asm_path_str = asm_path.as_path().display().to_string();
    let bin_path_str = asm_path
        .with_extension("asmgen")
        .as_path()
        .display()
        .to_string();

    // Create the assembly code
    let mut buffer = File::create(asm_path.as_path()).expect("need to success creating file");
    write(&asm, &mut buffer).unwrap();

    // Compile the assembly code
    if !Command::new("riscv64-linux-gnu-gcc")
        .args(["-static", &asm_path_str, "-o", &bin_path_str])
        .stderr(Stdio::null())
        .status()
        .unwrap()
        .success()
    {
        ::std::process::exit(SKIP_TEST);
    }

    // Emulate the executable
    let mut child = Command::new("qemu-riscv64-static")
        .args([&bin_path_str])
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to execute the compiled executable");

    let Some(status) = child
        .wait_timeout(Duration::from_millis(1000))
        .expect("failed to obtain exit status from child process")
    else {
        println!("timeout occurs");
        child.kill().unwrap();
        let _ = child.wait().unwrap();
        ::std::process::exit(SKIP_TEST);
    };

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
    drop(buffer);
    temp_dir.close().expect("temp dir deletion failed");

    println!(
        "kecc interp (expected): {}, qemu: {}",
        value as u8, qemu_status as u8
    );
    assert_eq!(value as u8, qemu_status as u8);
}

/// Tests end-to-end translation.
pub fn test_end_to_end(path: &Path) {
    // Check if the file has .c extension
    assert_eq!(path.extension(), Some(std::ffi::OsStr::new("c")));

    // Test parse
    let unit = Parse
        .translate(&path)
        .unwrap_or_else(|_| panic!("parse failed {}", path.display()));

    let file_path = path.display().to_string();
    let bin_path = path
        .with_extension("endtoend")
        .as_path()
        .display()
        .to_string();

    // Compile c file: If fails, test is vacuously success
    if !Command::new("clang")
        .args([
            "-fsanitize=float-divide-by-zero",
            "-fsanitize=undefined",
            "-fno-sanitize-recover=all",
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

    let _ = Command::new("rm")
        .arg(bin_path)
        .status()
        .expect("failed to remove compiled executable");

    let Some(status) = child
        .wait_timeout(Duration::from_millis(1000))
        .expect("failed to obtain exit status from child process")
    else {
        println!("timeout occurs");
        child.kill().unwrap();
        let _ = child.wait().unwrap();
        ::std::process::exit(SKIP_TEST);
    };

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

    let clang_status = some_or_exit!(status.code(), SKIP_TEST);

    // Execute optimized IR
    let mut ir = Irgen::default()
        .translate(&unit)
        .unwrap_or_else(|irgen_error| panic!("{}", irgen_error));
    let _ = O1::default().optimize(&mut ir);
    let args = Vec::new();
    let result = ir::interp(&ir, args).unwrap_or_else(|interp_error| panic!("{}", interp_error));
    // We only allow a main function whose return type is `int`
    let (value, width, is_signed) = result.get_int().expect("non-integer value occurs");
    assert_eq!(width, 32);
    assert!(is_signed);

    // When obtaining status from `clang` executable process, the status value is truncated to byte
    // size. For this reason, we make `fuzzer` generate the C source code which returns values
    // typecasted to `unsigned char`. However, during `creduce` to reduce the code, typecasting may
    // be nullified. So, we truncate the result value to byte size one more time here.
    println!(
        "clang (expected): {}, kecc interp: {}",
        clang_status as u8, value as u8
    );
    assert_eq!(clang_status as u8, value as u8);

    // Generate RISC-V assembly from IR
    let asm = Asmgen::default()
        .translate(&ir)
        .expect("fail to create riscv assembly code");

    let temp_dir = tempdir().expect("temp dir creation failed");
    let asm_path = temp_dir.path().join("temp.S");
    let asm_path_str = asm_path.as_path().display().to_string();
    let bin_path_str = asm_path
        .with_extension("asmgen")
        .as_path()
        .display()
        .to_string();

    // Create the assembly code
    let mut buffer = File::create(asm_path.as_path()).expect("need to success creating file");
    write(&asm, &mut buffer).unwrap();

    // Compile the assembly code
    if !Command::new("riscv64-linux-gnu-gcc")
        .args(["-static", &asm_path_str, "-o", &bin_path_str])
        .stderr(Stdio::null())
        .status()
        .unwrap()
        .success()
    {
        ::std::process::exit(SKIP_TEST);
    }

    // Emulate the executable
    let mut child = Command::new("qemu-riscv64-static")
        .args([&bin_path_str])
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to execute the compiled executable");

    let Some(status) = child
        .wait_timeout(Duration::from_millis(1000))
        .expect("failed to obtain exit status from child process")
    else {
        println!("timeout occurs");
        child.kill().unwrap();
        let _ = child.wait().unwrap();
        ::std::process::exit(SKIP_TEST);
    };

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
    drop(buffer);
    temp_dir.close().expect("temp dir deletion failed");

    println!(
        "clang (expected): {}, qemu: {}",
        clang_status as u8, qemu_status as u8
    );
    assert_eq!(clang_status as u8, qemu_status as u8);
}
