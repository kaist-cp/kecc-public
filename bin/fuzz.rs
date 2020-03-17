#[macro_use]
extern crate clap;
use clap::{crate_authors, crate_description, crate_version, App};

#[macro_use]
extern crate kecc;

use kecc::{Parse, Translate};

fn main() {
    let yaml = load_yaml!("fuzz_cli.yml");
    #[allow(deprecated)]
    let matches = App::from_yaml(yaml)
        .version(crate_version!())
        .about(crate_description!())
        .author(crate_authors!(", "))
        .get_matches();

    let input = matches.value_of("INPUT").unwrap();
    let unit = ok_or_exit!(Parse::default().translate(&input), 1);

    if matches.is_present("print") {
        kecc::write_c_test(&unit);
        return;
    }
}
