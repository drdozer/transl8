use std::io::{
    self,
    BufRead,
    Write
};


use clap::{
    Arg,
    App,
    crate_name,
    crate_version,
    crate_authors,
};

use chunks;

fn main() -> Result<(), io::Error> {
    let matches = App::new(crate_name!())
        .version(crate_version!())
        .author(crate_authors!())
        .about("Splits (large) input files up and execs child processes over these")
        .arg(Arg::with_name("type")
            .long("type")
            .multiple(false)
            .takes_value(true)
            .required(true)
            .help("Record format type. [embl|fasta]"))
        .arg(Arg::with_name("in")
            .short("i")
            .long("in")
            .multiple(true)
            .takes_value(true)
            .help("Input file. If not provided, defaults to STDIN."))
        .arg(Arg::with_name("commands")
            .multiple(true))
        .get_matches();
    
    let ins: Vec<Box<dyn BufRead>> =
        chunks::read_from_files_or_stin(matches.values_of("in"))?;
    
    let delim = match matches.value_of("type").unwrap() {
        "embl" => chunks::Delim::new(b">", false),
        "fasta" => chunks::Delim::new(b"//\n", true),
        unknown => {
            panic!("Unknown record type `{}'", unknown);
        }
    };

    let commands = matches.values_of("commands");

    for mut in_reader in ins {
        for chunk in chunks::chunks(in_reader, &delim) {
            println!("Chunk starting <<<");
            println!("{:#?}", chunk);
            println!("chunk ending >>>")
        }
    }

    Ok(())
}
