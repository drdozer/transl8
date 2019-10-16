use clap::{
    Arg,
    App,
    crate_name,
    crate_version,
    crate_authors,
};

use bio::{
    seq::{
        parser::LeadingColumns,
        fasta::*
    }
};

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
        .get_matches();
    

    let mut out =
        chunks::write_to_file_or_stdout(matches.value_of("seqOut"))
        .expect("Failed to open output file for writing");

    let ins = chunks::read_from_files_or_stdin(matches.values_of("seqIn"))
        .expect("Failed to open input file for reading");
    
    let delim = chunks::Delim::new(b"\n//\n", true);
    
    let fasta = FastaFormat::new();
    let embl_stanzas = LeadingColumns { tag_columns: 5, merge_tags: true };
    for in_reader in ins {
        for chunk in chunks::chunks(in_reader, &delim) {
            let chunk = chunk.expect("Failed to read chunk");
            let chunk_text = std::str::from_utf8(&chunk).unwrap();
            // println!("<<<");
            let stanzas = embl_stanzas.stanzas(chunk_text.lines()).collect::<Vec<_>>();
            // println!("Stanza view:");
            // println!("{:?}", stanzas);
            let id = stanzas.iter()
                .filter(|s| s.tag == Some("ID"))
                .flat_map(|s| s.lines.iter().flat_map(|l| l.split(';')))
                .next();
            // println!("ID line:");
            // println!("{:?}", id);
            let descr = stanzas.iter()
                .filter(|s| s.tag == Some("DE"))
                .map(|s| s.lines.iter()
                    .copied()
                    .collect::<String>()) // fixme: this may abolish whitespace between lines
                .next();
            // println!("Sequence:");
            let seq = stanzas.iter()
                .filter(|s| s.tag == Some("SQ"))
                .flat_map(|s| s.lines.iter()
                    .skip(1)
                    .flat_map(|l| l.split_whitespace()
                        .filter(|&t| t.chars().all(|c| c.is_ascii_alphabetic()))))
                .collect::<String>();
            // println!("{:?}", seq);
            if !seq.is_empty() {
                let descr_line = FastaRecord::descr_line(id, descr.as_ref().map(String::as_str));
                // println!("ID line text: {}", descr_line);
                let fasta_record = FastaRecord { descr_line, seq };
                fasta_record.write(&fasta, &mut out)
                    .expect("Problem writing fasta record out");
            }
            // println!(">>>")
        }
    }
}
