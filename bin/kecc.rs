use std::ffi::OsStr;
use std::path::Path;

use clap::Parser;

use lang_c::ast::TranslationUnit;

#[macro_use]
extern crate kecc;

use kecc::{
    ir, write, Asmgen, Deadcode, Gvn, IrParse, Irgen, Mem2reg, Optimize, Parse, SimplifyCfg,
    Translate, O1,
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
        Box::new(ok_or_exit!(::std::fs::File::open(output), 1))
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
