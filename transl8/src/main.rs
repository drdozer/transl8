
use std::io::{
    self,
    BufRead,
    Write
};

extern crate clap;
use clap::{
    Arg,
    App,
    crate_version,
    crate_authors,
};

use bio::{
    dna::*,
    seq::fasta::*,
};

use chunks;

fn main() -> Result<(), io::Error> {
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
    // todo - gzip support on input & output

    let matches = App::new("transl8")
        .version(crate_version!())
        .author(crate_authors!())
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

    let mut out =
        chunks::write_to_file_or_stdout(matches.value_of("seqOut"))?;

    let ins: Vec<Box<dyn BufRead>> =
        chunks::read_from_files_or_stin(matches.values_of("seqIn"))?;

    let fasta = FastaFormat::new();
    for mut in_reader in ins {
        let mut seq_txt = String::new();
        in_reader.read_to_string(&mut seq_txt)?;
        match parse_fastas(&seq_txt) {
            Ok((_, in_seqs)) => for (i, in_seq) in in_seqs.iter().enumerate() {
                let fd = FastaDescription::read(&in_seq.descr_line);
                write_6_phases(&fasta, &fd.identifier.unwrap_or_else(|| i.to_string()), &in_seq.seq, &mut out)?;
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