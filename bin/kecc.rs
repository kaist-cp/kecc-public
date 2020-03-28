#[macro_use]
extern crate clap;
use clap::{crate_authors, crate_description, crate_version, App};

#[macro_use]
extern crate kecc;

use kecc::{
    write, Asmgen, Deadcode, Gvn, Irgen, Mem2reg, Optimize, Parse, SimplifyCfg, Translate, O1,
};

fn main() {
    let yaml = load_yaml!("kecc_cli.yml");
    #[allow(deprecated)]
    let matches = App::from_yaml(yaml)
        .version(crate_version!())
        .about(crate_description!())
        .author(crate_authors!(", "))
        .get_matches();

    let input = matches.value_of("INPUT").unwrap();
    let unit = ok_or_exit!(Parse::default().translate(&input), 1);

    let output = matches.value_of("output").unwrap_or_else(|| "-");
    let mut output: Box<dyn ::std::io::Write> = if output == "-" {
        Box::new(::std::io::stdout())
    } else {
        Box::new(ok_or_exit!(::std::fs::File::open(output), 1))
    };

    if matches.is_present("parse") {
        return;
    }

    if matches.is_present("print") {
        write(&unit, &mut output).unwrap();
        return;
    }

    let mut ir = match Irgen::default().translate(&unit) {
        Ok(ir) => ir,
        Err(irgen_error) => {
            println!("{}", irgen_error);
            return;
        }
    };
    if matches.is_present("irgen") {
        write(&ir, &mut output).unwrap();
        return;
    }

    if matches.is_present("optimize") {
        O1::default().optimize(&mut ir);
    } else {
        if matches.is_present("simplify-cfg") {
            SimplifyCfg::default().optimize(&mut ir);
        }

        if matches.is_present("mem2erg") {
            Mem2reg::default().optimize(&mut ir);
        }

        if matches.is_present("deadcode") {
            Deadcode::default().optimize(&mut ir);
        }

        if matches.is_present("gvn") {
            Gvn::default().optimize(&mut ir);
        }
    }

    let asm = ok_or_exit!(Asmgen::default().translate(&ir), 1);
    write(&asm, &mut output).unwrap();
}
