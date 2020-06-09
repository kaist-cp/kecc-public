#[macro_use]
extern crate clap;
use clap::{crate_authors, crate_description, crate_version, App};

extern crate kecc;

use std::path::Path;

fn main() {
    let yaml = load_yaml!("fuzz_cli.yml");
    #[allow(deprecated)]
    let matches = App::from_yaml(yaml)
        .version(crate_version!())
        .about(crate_description!())
        .author(crate_authors!(", "))
        .get_matches();

    let input = matches.value_of("INPUT").unwrap();

    if matches.is_present("print") {
        kecc::test_write_c(Path::new(&input));
        return;
    }

    if matches.is_present("irgen") {
        kecc::test_irgen(Path::new(&input));
        return;
    }

    if matches.is_present("irparse") {
        kecc::test_irparse(Path::new(&input));
        return;
    }

    if matches.is_present("simplify-cfg") {
        todo!("test simplify-cfg");
    }

    if matches.is_present("mem2erg") {
        todo!("test mem2reg");
    }

    if matches.is_present("deadcode") {
        todo!("test deadcode");
    }

    if matches.is_present("gvn") {
        todo!("test gvn");
    }

    if matches.is_present("end-to-end") {
        kecc::test_end_to_end(Path::new(&input));
        return;
    }

    assert_eq!(
        Path::new(input).extension(),
        Some(std::ffi::OsStr::new("ir"))
    );
    kecc::test_asmgen(Path::new(&input));
}
