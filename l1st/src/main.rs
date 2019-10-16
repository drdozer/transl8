use clap::{
    Arg,
    App,
    crate_name,
    crate_version,
    crate_authors,
};

use std::io::Write;
use std::io;

use bio::seq::fasta::*;


fn main() -> Result<(), io::Error> {
    let matches = App::new(crate_name!())
        .version(crate_version!())
        .author(crate_authors!())
        .about("List sequence entry IDs")
        .arg(Arg::with_name("seqIn")
            .short("i")
            .long("seqIn")
            .multiple(false)
            .takes_value(true)
            .help("Fasta-formatted sequence input file. If not provided, defaults to STDIN.")
        )
        .arg(Arg::with_name("idsOut")
            .short("o")
            .long("idsOut")
            .multiple(false)
            .takes_value(true)
            .help("Id list output file. If not provided, defaults to STDOUT."))
        .get_matches();
    
    let mut out =
        chunks::write_to_file_or_stdout(matches.value_of("seqOut"))
        .expect("Failed to open output file for writing");

    let ins = chunks::read_from_files_or_stdin(matches.values_of("seqIn"))
        .expect("Failed to open input file for reading");

    let delim = chunks::Delim::new(b">", false);

    for in_reader in ins {
        for chunk in chunks::chunks(in_reader, &delim) {
            let chunk = chunk.expect("Failed to read chunk");
            let chunk_text = std::str::from_utf8(&chunk).unwrap();
            match parse_fastas(&chunk_text) {
                Ok((_, in_seqs)) => for in_seq in in_seqs {
                    let fd = FastaDescription::read(&in_seq.descr_line);
                    match fd.identifier {
                        Some(id) => {
                            writeln!(out, "{}", id)?
                        }
                        None => ()
                    }
                },
                Err(e) => println!("Error parsing fasta input:\n{:#?}\n", e)
            }
        }
    }

    Ok(())
}
