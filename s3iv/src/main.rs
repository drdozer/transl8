
use std::io::{
    self,
    BufRead,
    BufReader,
    Write
};
use std::fs::File;

extern crate clap;
use clap::{
    Arg,
    App,
    crate_version,
    crate_authors,
};

use bio::{
    dna::*,
    seq::*,
};

fn main() -> Result<(), io::Error> {

    let matches = App::new("s3iv")
        .version(crate_version!())
        .author(crate_authors!())
        .about("Seivs (filters) fasta files")
        .arg(Arg::with_name(""))
        .arg(Arg::with_name("seqIn")
            .short("i")
            .long("seqIn")
            .multiple(true)
            .takes_value(true)
            .help("Sequence input file. If not provided, defaults to STDIN."))
        .arg(Arg::with_name("seqOut")
            .short("o")
            .long("seqOut")
            .multiple(false)
            .takes_value(true)
            .help("Sequence output file. If not provided, defaults to STDOUT."))
        .arg(Arg::with_name("minLength")
            .short("m")
            .long("minLength")
            .multiple(false)
            .takes_value(true)
            .required(false)
            .help("Minimum sequence length. By default, no sequences are rejected for being too short."))
        .arg(Arg::with_name("minLength")
            .short("x")
            .long("maxLength")
            .multiple(false)
            .takes_value(true)
            .required(false)
            .help("Maximum sequence length. By default, no sequences are rejected for being too long."))
        .arg(Arg::with_name("polyN")
            .short("n")
            .long("polyN")
            .multiple(false)
            .takes_value(false)
            .required(false)
            .help("Enable filtering out of poly-n sequences."))
        .get_matches();


    let mut out =
        bio::writeToFileOrStdout(matches.value_of("seqOut"))?;

    let ins: Vec<Box<dyn BufRead>> =
        bio::readFromFilesOrStin(matches.values_of("seqIn"))?;

    let fasta = FastaFormat::new();

    let filter_n = if matches.is_present("polyN") {
        accept
    } else {
        seiv_n
    }

    let filter_short = match matches.value_of("minLength") {
        Some(m) => seiv_min(m.parse().unwrap()),
        None => accept
    };

    Ok(())
}

fn accept<T>(t: T) -> bool { true }
fn reject<T>(t: T) -> bool { false }

fn seiv_n(fasta: FastaRecord) -> bool {
    fasta.seq.chars().all(|c| c == 'n' || c == 'N')
}

fn seiv_min(l: usize) -> impl Fn(FastaRecord) -> bool {
    |fastaRecord| fastaRecord.seq.len() < l
}