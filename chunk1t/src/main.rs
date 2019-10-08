use std::io::{
    self,
    BufRead
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
        "fasta" => chunks::Delim::new(b">", false),
        "embl" => chunks::Delim::new(b"\n//\n", true),
        unknown => {
            panic!("Unknown record type `{}'", unknown);
        }
    };

    //let commands = matches.values_of("commands");

    for in_reader in ins {
        for chunk in chunks::chunks(in_reader, &delim) {
            print_chunk(chunk);
        }
    }

    Ok(())
}

fn print_chunk(chunk: Result<Vec<u8>, std::io::Error>) {
    println!("Chunk starting <<<");
    println!("{}", std::str::from_utf8(&chunk.unwrap()[..]).unwrap().trim());
    println!("chunk ending >>>")
}