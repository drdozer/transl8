#[macro_use]
extern crate lazy_static;

use std::io::{
    self,
    BufRead,
    BufReader,
    Write
};
use std::fs::File;



pub mod dna;
pub mod seq;

pub fn write_to_file_or_stdout(out: Option<&str>) -> io::Result<Box<dyn Write>> {
    let writer: Box<dyn Write> = match out {
       None => Box::new(std::io::stdout()),
       Some(seq_out) => Box::new(File::create(seq_out)?)
    };
    Ok(writer)
}

pub fn read_from_files_or_stin<'a, I>(ins: Option<I>) -> io::Result<Vec<Box<dyn BufRead>>>
where
    I: Iterator<Item = &'a str>
{
    let brs: Vec<Box<dyn BufRead>> = match ins {
        Some(in_names) => {
            let x = in_names
                .map(|i| {
                    let f = File::open(i).unwrap(); // todo: we should be returning this, not panicking
                    let b: Box<dyn BufRead> = Box::new(BufReader::new(f));
                    b
                });
            x.collect()
        },
                
        None =>
            vec![Box::new(BufReader::new(std::io::stdin()))],
    };
    Ok(brs)
}