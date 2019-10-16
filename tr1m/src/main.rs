
use std::io::Write;
use std::io;
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
use bio::seq::gff3::{GffRecord, OneBased};

use chunks;

fn main() -> Result<(), io::Error> {
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
        .arg(Arg::with_name("mapping")
            .short("m")
            .long("mapping")
            .multiple(false)
            .takes_value(true)
            .help("Name of mapping file documenting the raw and clipped identifiers. Only generates mapping file if supplied."))
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
        txt.lines().filter(|l| !l.starts_with("#"))
            .map(|l| FromStr::from_str(l)
            .expect("Failed to parse gff line"))
            .collect()
    };

    let mut mapping = matches.value_of("mapping").map(|m| fs::File::create(m)
        .expect("Problem opening mapping file for writing"));

    let mut write_mapping = move |from: &str, to: &str| mapping.iter_mut().for_each(
        |f| writeln!(f, "{}\t{}", from, to).expect("Unable to write mapping pair to file"));


    let delim = chunks::Delim::new(b">", false);
    let fasta = FastaFormat::new();
    for in_reader in ins {
        for chunk in chunks::chunks(in_reader, &delim) {
            let chunk = chunk.expect("Failed to read chunk");
            let chunk_text = std::str::from_utf8(&chunk).unwrap();
            match parse_fastas(&chunk_text) {
                Ok((_, in_seqs)) => for in_seq in in_seqs {
                    let fd = FastaDescription::read(&in_seq.descr_line);
                    match fd.identifier {
                        Some(id) => {
                            // println!("Got fasta with id {:?}", id);
                            let clps = gff.iter()
                                .filter(|g| g.seq_id == id && g.start == OneBased::new(1))
                                .map(|g| g.end.at())
                                .max();
                            match clps {
                                Some(clip) => {
                                    // println!("Got fasta entry with id {} and clip {}. Writing unchanged.", id, clip);
                                    let clipped_id = format!("{}_clipped_{}", id, clip);
                                    let descr_line = FastaRecord::descr_line(Some(&clipped_id), fd.description.as_ref().map(String::as_ref));
                                    let clipped_seq = in_seq.seq[(clip as usize)..].to_string();
                                    let clipped_rec = FastaRecord { descr_line, seq: clipped_seq };
                                    // in_seq.write(&fasta, &mut out)?;
                                    write_mapping(&id, &clipped_id);
                                    clipped_rec.write(&fasta, &mut out)?;
                                }
                                None => {
                                    // println!("Got fasta entry with id {} but no clip. Writing unchanged.", id);
                                    write_mapping(&id, &id);
                                    in_seq.write(&fasta, &mut out)?;
                                }
                            }
                        }
                        None => {
                            // println!("Got fasta entry with no identifier. Writing unchanged.");
                            in_seq.write(&fasta, &mut out)?;
                        }
                    }
                },
                Err(e) => println!("Error parsing fasta input:\n{:#?}\n", e)
            }
        }
    }

    Ok(())
}
