#[macro_use]
extern crate lazy_static;

use std::io::{
    self,
    BufRead,
    BufReader,
    Write
};
use std::fs::File;

extern crate clap;
use clap::{Arg, App};

extern crate nom;

mod dna;
use dna::*;

mod seq;
use seq::*;

fn main() -> Result<(), io::Error> {
    // todo - read a stream of fasta files
    // todo - 6 frame translation of each entry
    // todo - write all 6 out to the output stream
    //
    // todo - toggle for
    //    single output per frame
    //    separate outupt broken by stop
    //    separate output broken by start-stop (orf mode)
    //
    // todo - min/max length filter
    //
    // todo - render associated DNA sequences
    // todo - render coordinates of translated regions as GFF
    //
    // todo - command line opts to read file vs url vs stdin
    // todo - command line opts to write to file vs stdout vs post to url
    //
    // todo - gzip support on input & output

    let matches = App::new("transl8")
        .version("0.1")
        .author("Matthew Pocock <turingatemyhamster@gmail.com>")
        .about("Performs 6-frame translation on DNA sequences")
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
        .get_matches();

    let mut out: Box<dyn Write> = match matches.value_of("seqOut") {
        None => Box::new(std::io::stdout()),
        Some(seq_out) => Box::new(File::create(seq_out)?)
    };

    let ins: Vec<Box<dyn BufRead>> = match matches.values_of("seqIn") {
        Some(seq_ins) =>
            seq_ins
                .map(|i| {
                    let b: Box<dyn BufRead> = Box::new(BufReader::new(File::open(i).unwrap()));
                    b
                })
                .collect(),
        None =>
            vec![Box::new(BufReader::new(std::io::stdin()))],
    };


    let fasta = FastaFormat::new();
    for mut in_reader in ins {
        let mut seq_txt = String::new();
        in_reader.read_to_string(&mut seq_txt)?;
        match parse_fastas(&seq_txt) {
            Ok((_, in_seqs)) => for (i, in_seq) in in_seqs.iter().enumerate() {
                let fd = FastaDescription::read(&in_seq.descr_line);
                write_6_phases(&fasta, &fd.identifier.unwrap_or(i.to_string()), &in_seq.seq, &mut out)?;
            },
            Err(e) => println!("Error parsing fasta input:\n{:#?}\n", e)
        }
    }

    Ok(())
}

fn write_6_phases<W : Write>(fasta: &FastaFormat, id: &str, dna_str: &str, out: &mut W) -> Result<(), io::Error> {
    let lc_dna = &dna_str.to_lowercase()[..];
    let rev_cmp = reverse_complement(lc_dna);

    let phase0 = FastaRecord { 
        descr_line: format!("{}_phase_0", id),
        seq: translate(&frame(lc_dna, 0))
    };
    let phase1 = FastaRecord { 
        descr_line: format!("{}_phase_1", id),
        seq: translate(&frame(lc_dna, 1))
    };
    let phase2 = FastaRecord { 
        descr_line: format!("{}_phase_2", id),
        seq: translate(&frame(lc_dna, 2))
    };
    let phase3 = FastaRecord {
        descr_line: format!("{}_phase_3", id),
        seq: translate(&frame(&rev_cmp, 3))
    };
    let phase4 = FastaRecord {
        descr_line: format!("{}_phase_4", id),
        seq: translate(&frame(&rev_cmp, 4))
    };
    let phase5 = FastaRecord {
        descr_line: format!("{}_phase_5", id),
        seq: translate(&frame(&rev_cmp, 5))
    };

    phase0.write(&fasta, out)?;
    phase1.write(&fasta, out)?;
    phase2.write(&fasta, out)?;
    phase3.write(&fasta, out)?;
    phase4.write(&fasta, out)?;
    phase5.write(&fasta, out)?;

    Ok(())
}