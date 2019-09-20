#[macro_use]
extern crate lazy_static;


mod dna;
use dna::*;

mod seq;
use seq::*;

fn main() {
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

    println!("Hello, world!");

    let dna_str = "tattcgccgtagtcagttatatatgcgtcgtactgcgcgcgctatgacta";
    println!("Got DNA: {}", dna_str);

    let rev_comp: String = reverse_complement(dna_str);

    println!("Reverse complement DNA: {}", rev_comp);

    println!("Phase + 0 {:?}", frame(dna_str, 0));
    println!("Phase + 1 {:?}", frame(dna_str, 1));
    println!("Phase + 2 {:?}", frame(dna_str, 2));
    println!("Phase - 0 {:?}", frame(&rev_comp, 0));
    println!("Phase - 1 {:?}", frame(&rev_comp, 1));
    println!("Phase - 2 {:?}", frame(&rev_comp, 2));

    println!("Phase + 0 translation {:?}", translate(&frame(dna_str, 0)));


    let fasta = FastaFormat::new();

    let phase0 = FastaRecord { 
        descrLine: "phase_0",
        seq: &translate(&frame(dna_str, 0))
    };
    let phase1 = FastaRecord { 
        descrLine: "phase_1",
        seq: &translate(&frame(dna_str, 1))
    };
    let phase2 = FastaRecord { 
        descrLine: "phase_2",
        seq: &translate(&frame(dna_str, 2))
    };
    let phase3 = FastaRecord {
        descrLine: "phase_3",
        seq: &translate(&frame(&rev_comp, 3))
    };
    let phase4 = FastaRecord {
        descrLine: "phase_4",
        seq: &translate(&frame(&rev_comp, 4))
    };
    let phase5 = FastaRecord {
        descrLine: "phase_5",
        seq: &translate(&frame(&rev_comp, 5))
    };

    phase0.write(&fasta, &mut std::io::stdout());
    phase1.write(&fasta, &mut std::io::stdout());
    phase2.write(&fasta, &mut std::io::stdout());
    phase3.write(&fasta, &mut std::io::stdout());
    phase4.write(&fasta, &mut std::io::stdout());
    phase5.write(&fasta, &mut std::io::stdout());
    
}