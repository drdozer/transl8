
use std::io::{
    self,
    BufRead,
};

extern crate clap;
use clap::{
    Arg,
    App,
    crate_version,
    crate_authors,
};

use bio::{
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

    // true if the sequence is all N, false otherwise
    fn seiv_n(fasta: &FastaRecord) -> bool {
        fasta.seq.chars().all(|c| c == 'n' || c == 'N')
    }
    let filter_n = |fr: &FastaRecord| (matches.is_present("polyN") &&
        seiv_n(fr));

    // true if the sequence is too short
    let short = matches.value_of("minLength")
        .iter()
        .flat_map(|m| m.parse::<usize>().ok())
        .next();
    let filter_short = |fr : &FastaRecord| match short {
        Some(m) => fr.seq.len() < m,
        None    => false
    };

    // true if the sequence is too long
    let long = matches.value_of("maxLength")
        .iter()
        .flat_map(|m| m.parse::<usize>().ok())
        .next();
    let filter_long  = |fr : &FastaRecord| match long {
        Some(x) => fr.seq.len() > x,
        None    => false
    };

    let reject_fasta = |fr: &FastaRecord| {
        let is_n = filter_n(&fr);
        let is_short = filter_short(&fr);
        let is_long = filter_long(&fr);
        is_n || is_short || is_long
    };

    let fasta = FastaFormat::new();
    for mut in_reader in ins {
        let mut seq_txt = String::new();
        in_reader.read_to_string(&mut seq_txt)?;
        match parse_fastas(&seq_txt) {
            Ok((_, in_seqs)) => {
                 /* without explicit lambda, fr became &&fr */
                let filtered = in_seqs.iter().filter(|fr| !reject_fasta(*fr));
                for in_seq in filtered {
                    in_seq.write(&fasta, &mut out)?;
                }
            },
            Err(e) => println!("Error parsing fasta input:\n{:#?}\n", e)
        }
    }

    Ok(())
}

