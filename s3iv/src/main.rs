
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

    let filter_n = |&fr| if matches.is_present("polyN") {
        seiv_n(fr)
    } else {
        accept(fr)
    };

    let short = matches.value_of("minLength")
        .iter()
        .flat_map(|m| m.parse::<usize>().ok())
        .next();
    let filter_short = |fr : &FastaRecord| match short {
        Some(m) => fr.seq.len() >= m,
        None    => true
    };

    let long = matches.value_of("maxLength")
        .iter()
        .flat_map(|m| m.parse::<usize>().ok())
        .next();
    let filter_long  = |fr : &FastaRecord| match long {
        Some(x) => fr.seq.len() <= x,
        None    => true
    };

    let filter = |fr| filter_n(fr) || filter_short(fr) || filter_long(fr);

    Ok(())
}


type FastaRecordPred = Box<dyn Fn(&FastaRecord) -> bool>;

fn accept<T>(t: &T) -> bool { true }

fn seiv_n(fasta: &FastaRecord) -> bool {
    let all_n = fasta.seq.chars().all(|c| c == 'n' || c == 'N');
    !all_n
}
