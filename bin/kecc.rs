use std::ffi::OsStr;
use std::path::Path;

use clap::{crate_authors, crate_description, crate_version, load_yaml, App};

use lang_c::ast::TranslationUnit;

#[macro_use]
extern crate kecc;

use kecc::{
    ir, write, Asmgen, Deadcode, Gvn, IrParse, Irgen, Mem2reg, Optimize, Parse, SimplifyCfg,
    Translate, O1,
};

fn main() {
    let yaml = load_yaml!("kecc_cli.yml");
    #[allow(deprecated)]
    let matches = App::from(yaml)
        .version(crate_version!())
        .about(crate_description!())
        .author(crate_authors!(", "))
        .get_matches();

    let input = matches.value_of("INPUT").unwrap();
    let input = Path::new(input);

    let output = matches.value_of("output").unwrap_or("-");
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

fn compile_c(
    input: &TranslationUnit,
    output: &mut dyn ::std::io::Write,
    matches: &clap::ArgMatches,
) {
    if matches.is_present("parse") {
        return;
    }

    if matches.is_present("print") {
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

    if matches.is_present("irgen") {
        write(&ir, output).unwrap();
        return;
    }

    compile_ir(&mut ir, output, matches)
}

fn compile_ir(
    input: &mut ir::TranslationUnit,
    output: &mut dyn ::std::io::Write,
    matches: &clap::ArgMatches,
) {
    if matches.is_present("irparse") {
        return;
    }

    if matches.is_present("irprint") {
        write(input, output).unwrap();
        return;
    }

    if matches.is_present("optimize") {
        O1::default().optimize(input);
    } else {
        if matches.is_present("simplify-cfg") {
            SimplifyCfg::default().optimize(input);
        }

        if matches.is_present("mem2reg") {
            Mem2reg::default().optimize(input);
        }

        if matches.is_present("deadcode") {
            Deadcode::default().optimize(input);
        }

        if matches.is_present("gvn") {
            Gvn::default().optimize(input);
        }
    }

    if matches.is_present("iroutput") {
        write(input, output).unwrap();
        return;
    }

    if matches.is_present("irrun") {
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
