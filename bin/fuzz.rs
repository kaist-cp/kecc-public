use std::path::Path;

use clap::Parser;

#[derive(Debug, Parser)]
#[clap(name = "fuzz", version, author, about)]
struct FuzzCli {
    /// Fuzzes C AST Printer
    #[clap(short, long)]
    print: bool,

    /// Fuzzes irgen
    #[clap(short, long)]
    irgen: bool,

    /// Fuzzes irparse
    #[clap(long)]
    irparse: bool,

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

    /// Fuzzes irgen, optimize and asmgen pipeline
    #[clap(long = "end-to-end")]
    end_to_end: bool,

    /// Sets the input file to use
    input: String,
}

fn main() {
    let matches = FuzzCli::parse();
    let input = matches.input;

    if matches.print {
        kecc::test_write_c(Path::new(&input));
        return;
    }

    if matches.irgen {
        kecc::test_irgen(Path::new(&input));
        return;
    }

    if matches.irparse {
        kecc::test_irparse(Path::new(&input));
        return;
    }

    if matches.simplify_cfg {
        todo!("test simplify-cfg");
    }

    if matches.mem2reg {
        todo!("test mem2reg");
    }

    if matches.deadcode {
        todo!("test deadcode");
    }

    if matches.gvn {
        todo!("test gvn");
    }

    if matches.end_to_end {
        kecc::test_end_to_end(Path::new(&input));
        return;
    }

    assert_eq!(
        Path::new(&input).extension(),
        Some(std::ffi::OsStr::new("ir"))
    );
    kecc::test_asmgen(Path::new(&input));
}
