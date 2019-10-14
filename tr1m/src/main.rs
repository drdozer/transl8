
use std::io::{
    self,
    BufRead,
};
use std::str::FromStr;
use std::fs;

extern crate clap;
use clap::{
    Arg,
    App,
    crate_name,
    crate_version,
    crate_authors,
};

use bio::seq::fasta::*;
use bio::seq::gff3::GffRecord;

use chunks;

fn main() {
    let matches = App::new(crate_name!())
        .version(crate_version!())
        .author(crate_authors!())
        .about("Converts embl files to fasta files")
        .arg(Arg::with_name("seqIn")
            .short("i")
            .long("seqIn")
            .multiple(false)
            .takes_value(true)
            .help("EMBL-formatted sequence input file. If not provided, defaults to STDIN.")
        )
        .arg(Arg::with_name("seqOut")
            .short("o")
            .long("seqOut")
            .multiple(false)
            .takes_value(true)
            .help("Sequence output file. If not provided, defaults to STDOUT."))
        .arg(Arg::with_name("gff")
            .short("g")
            .long("gff")
            .multiple(false)
            .takes_value(true)
            .help("GFF3 file containing regions to clip"))
        .get_matches();

    let mut out =
        chunks::write_to_file_or_stdout(matches.value_of("seqOut"))
        .expect("Failed to open output file for writing");

    let ins = chunks::read_from_files_or_stdin(matches.values_of("seqIn"))
        .expect("Failed to open input file for reading");

    let gff: Vec<GffRecord> = {
        let gff_file_name = matches.value_of("gff")
            .expect("Must provide a gff file");
        let txt = fs::read_to_string(gff_file_name).expect("Could not read the gff file");
        let lns = txt.lines().filter(|l| !l.starts_with("#")).map(|l| FromStr::from_str(l).expect("Failed to parse gff line")).collect();
        lns
    };



    let fasta = FastaFormat::new();

}
