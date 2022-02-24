use clap::Parser;

use std::ffi::OsStr;
use std::io::Write;
use std::os::unix::io::{FromRawFd, IntoRawFd};
use std::path::Path;
use std::process::{Command, Stdio};

use lang_c::ast::TranslationUnit;
use tempfile::tempdir;

use kecc::{
    ir, ok_or_exit, write, Asmgen, Deadcode, Gvn, IrParse, IrVisualizer, Irgen, Mem2reg, Optimize,
    Parse, SimplifyCfg, Translate, O1,
};

#[derive(Debug, Parser)]
#[clap(name = "kecc", version, author, about)]
struct KeccCli {
    /// Parses the input C file
    #[clap(long)]
    parse: bool,

    /// Prints the input AST
    #[clap(short, long)]
    print: bool,

    /// Generates IR
    #[clap(short, long)]
    irgen: bool,

    /// Parses the input IR file
    #[clap(long)]
    irparse: bool,

    /// Prints the input IR AST
    #[clap(long)]
    irprint: bool,

    /// Executes the input file
    #[clap(long)]
    irrun: bool,

    /// Visualizes IR
    #[clap(long, value_name = "FILE")]
    irviz: Option<String>,

    /// Optimizes IR
    #[clap(short = 'O', long)]
    optimize: bool,

    /// Performs simplify-cfg
    #[clap(long = "simplify-cfg")]
    simplify_cfg: bool,

    /// Performs mem2reg
    #[clap(long)]
    mem2reg: bool,

    /// Performs deadcode elimination
    #[clap(long)]
    deadcode: bool,

    /// Performs gvn
    #[clap(long)]
    gvn: bool,

    /// Prints the output IR
    #[clap(long)]
    iroutput: bool,

    /// Sets the output file to use
    #[clap(short, long, value_name = "FILE")]
    output: Option<String>,

    /// Sets the input file to use
    input: String,
}

fn main() {
    let matches = KeccCli::parse();
    let input = Path::new(&matches.input);

    let output = matches.output.clone().unwrap_or_else(|| "-".to_string());

    let mut output: Box<dyn ::std::io::Write> = if output == "-" {
        Box::new(::std::io::stdout())
    } else {
        Box::new(ok_or_exit!(::std::fs::File::create(output), 1))
    };

    let ext = input.extension();
    if ext == Some(OsStr::new("c")) {
        let input = ok_or_exit!(Parse::default().translate(&input), 1);
        compile_c(&input, &mut output, &matches);
    } else if ext == Some(OsStr::new("ir")) {
        let mut input = ok_or_exit!(IrParse::default().translate(&input), 1);
        compile_ir(&mut input, &mut output, &matches);
    } else {
        panic!("Unsupported file extension: {:?}", ext);
    }
}

fn compile_c(input: &TranslationUnit, output: &mut dyn ::std::io::Write, matches: &KeccCli) {
    if matches.parse {
        return;
    }

    if matches.print {
        write(input, output).unwrap();
        return;
    }

    let mut ir = match Irgen::default().translate(input) {
        Ok(ir) => ir,
        Err(irgen_error) => {
            println!("{}", irgen_error);
            return;
        }
    };

    if matches.irgen {
        write(&ir, output).unwrap();
        return;
    }

    compile_ir(&mut ir, output, matches)
}

fn compile_ir(
    input: &mut ir::TranslationUnit,
    output: &mut dyn ::std::io::Write,
    matches: &KeccCli,
) {
    if matches.irparse {
        return;
    }

    if matches.irprint {
        write(input, output).unwrap();
        return;
    }

    if let Some(path) = &matches.irviz {
        assert_eq!(
            Path::new(&path).extension(),
            Some(std::ffi::OsStr::new("png"))
        );
        let img_path = Path::new(path);
        let dot = IrVisualizer::default()
            .translate(input)
            .expect("ir visualize failed");

        let temp_dir = tempdir().expect("temp dir creation failed");
        let dot_path = temp_dir.path().join("temp.dot");
        let dot_path_str = dot_path.as_path().display().to_string();
        let img_path_str = img_path.display().to_string();

        // Create the dot file
        let mut buffer =
            ::std::fs::File::create(dot_path.as_path()).expect("need to success creating file");
        buffer
            .write(dot.as_bytes())
            .expect("failed to write to dot file");

        // Create the image file
        let img = ::std::fs::File::create(&img_path_str).expect("need to success creating file");

        // Translate dot file into image
        if !Command::new("dot")
            .args(&["-Tpng", &dot_path_str])
            .stdout(unsafe { Stdio::from_raw_fd(img.into_raw_fd()) })
            .status()
            .unwrap()
            .success()
        {
            panic!("failed to save image file");
        }

        drop(buffer);
        temp_dir.close().expect("temp dir deletion failed");
    }

    if matches.optimize {
        O1::default().optimize(input);
    } else {
        if matches.simplify_cfg {
            SimplifyCfg::default().optimize(input);
        }

        if matches.mem2reg {
            Mem2reg::default().optimize(input);
        }

        if matches.deadcode {
            Deadcode::default().optimize(input);
        }

        if matches.gvn {
            Gvn::default().optimize(input);
        }
    }

    if matches.iroutput {
        write(input, output).unwrap();
        return;
    }

    if matches.irrun {
        let result = ir::interp(input, Vec::new()).unwrap();
        let (value, width, is_signed) = result.get_int().expect("non-integer value occurs");
        assert_eq!(width, 32);
        assert!(is_signed);

        // When obtain status from `gcc` executable process, status value is truncated to byte size.
        // So, we also truncate result value to byte size before printing it.
        println!("[result] {:?}", value as u8);
        return;
    }

    let asm = ok_or_exit!(Asmgen::default().translate(input), 1);
    write(&asm, output).unwrap();
}
