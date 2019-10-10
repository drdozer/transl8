use std::io::{self, BufRead, BufReader, Error, ErrorKind, Write};
use std::fs::File;

fn first_index_of(needle: &[u8], haystack: &[u8]) -> Option<usize> {
    // println!("Searching for a needle of length {} in a haystack of length {}", needle.len(), haystack.len());
    if needle.len() > haystack.len() {
        None
    } else {
        for i in 0..= (haystack.len() - needle.len()) {
            if haystack[i..].iter().zip(needle).all(|(n, h)| n == h) {
                return Some(i);
            }
        }

        None
    }
}

fn extend_buffer<R>(buf: &mut Vec<u8>, reader: &mut R) -> Result<usize, Error>
where
    R: BufRead,
{
    let read = loop {
        match reader.fill_buf() {
            Ok(r) => break r,
            Err(ref e) if e.kind() == ErrorKind::Interrupted => continue,
            Err(e) => return Err(e),
        }
    };
    let l = read.len();
    buf.extend_from_slice(read);
    reader.consume(l);
    Ok(l)
}

fn compact_buf(from: usize, buf: &mut Vec<u8>) -> usize {
    if from != 0 {
        buf.copy_within(from.., 0);
        buf.truncate(buf.len() - from);
        from
    } else {
        0
    }
}

pub fn chunks<'a, R : Sized>(reader: R, delim: &'a Delim) -> Chunker<'a, R>
{
    Chunker {
        reader,
        sentinel: delim.sentinel,
        marks_chunk_end: delim.marks_chunk_end,
        buf: Vec::new(),
        from: 0,
        searched: 0,
        done: false,
    }
}

pub struct Delim<'a> {
    sentinel: &'a [u8],
    marks_chunk_end: bool,
}

impl Delim<'_> {
    pub fn new(sentinel: &[u8], marks_chunk_end: bool) -> Delim<'_> {
        Delim { sentinel, marks_chunk_end }
    }
}

pub struct Chunker<'a, R> {
    reader: R,
    sentinel: &'a [u8],
    marks_chunk_end: bool,
    buf: Vec<u8>,
    from: usize,
    searched: usize,
    done: bool,
}

impl<'a, R> Iterator for Chunker<'a, R>
where
    R: BufRead,
{
    type Item = Result<Vec<u8>, Error>;
    fn next(&mut self) -> Option<Result<Vec<u8>, Error>> {
        if self.done {
            None
        } else {
            loop {
                // an empty buffer - this is (like) starting from the beginning
                if self.buf.is_empty() {
                    // println!("Extending buffer");
                    match extend_buffer(&mut self.buf, &mut self.reader) {
                        Ok(0) => {
                            // println!("Zero length buffer extension. Assuming EOF");
                            self.done = true;
                            return None;
                        }
                        Ok(_len) => { 
                            // println!("Extended buffer by {} bytes", len);
                            continue
                        }
                        Err(e) => {
                            // println!("Some error occured: {:?}", e);
                            return Some(Err(e));
                        }
                    }
                } else {
                    //println!("Using existing buffer from: {} searched: {} length: {}", self.from, self.searched, self.buf.len());
                    match first_index_of(self.sentinel, &self.buf[self.searched..]) {
                        None => {
                            let cmp = compact_buf(self.from, &mut self.buf);

                            match extend_buffer(&mut self.buf, &mut self.reader) {
                                Ok(0) => {
                                    self.from = self.buf.len();
                                    self.done = true;
                                    return Some(Ok(self.buf[..].to_vec()));
                                }
                                Ok(_len) => {
                                    self.from = 0;
                                    self.searched -= cmp;
                                    continue;
                                }
                                Err(e) => return Some(Err(e)),
                            }
                        }
                        Some(p) => {
                            // p is a coordinate within self.buf[self.searched..]
                            let sentinel_start = p + self.searched; // now it is within the current buf
                            let sentinel_end = sentinel_start + self.sentinel.len();
                            
                            // the hit ends at
                            let hit_end = if self.marks_chunk_end {
                                sentinel_end
                            } else {
                                sentinel_start
                            };

                            let hit = &self.buf[self.from..hit_end];
                            self.from = hit_end;
                            self.searched = sentinel_end;
                            if !hit.is_empty()  { return Some(Ok(hit.to_vec())) }
                        }
                    }

                }
            }
        }
    }
}


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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_first_index_of() {
        assert_eq!(first_index_of(b"x", b""), None);
        assert_eq!(first_index_of(b"x", b"x"), Some(0));
        assert_eq!(first_index_of(b"la", b"this is largely rubbish"), Some(8));
    }

    fn fasta_delim() -> Delim<'static> { Delim::new(b">", false) }
    fn embl_delim() -> Delim<'static> { Delim::new(b"//\n", true) }

    #[test]
    fn test_empty() {
        
        let delim = fasta_delim();
        let mut ci = chunks(std::io::empty(), &delim);
        match ci.next() {
            None => {},
            Some(e) => panic!("Expected no chunks from an empty input, but got {:?}", e)
        }
    }

    #[test]
    fn test_one_undelimited_marks_start() {
        let delim = fasta_delim();
        let input: &[u8] = b"some\nstuff\ngoes\nhere";
        let mut ci = chunks(input, &delim);
        match ci.next() {
            None => panic!("Expected a chunk but got none"),
            Some(Ok(c)) => assert_eq!(c, input),
            Some(e) => panic!("Got an unexpected error: {:?}", e)
        }    
    }

    #[test]
    fn test_one_undelimited_marks_end() {
        let delim = embl_delim();
        let input: &[u8] = b"some\nstuff\ngoes\nhere";
        let mut ci = chunks(input, &delim);
        match ci.next() {
            None => panic!("Expected a chunk but got none"),
            Some(Ok(c)) => assert_eq!(c, input),
            Some(e) => panic!("Got an unexpected error: {:?}", e)
        }    
    }

    #[test]
    fn test_one_delimited_marks_start() {
        let delim = fasta_delim();
        let input: &[u8] = b">some\nstuff\ngoes\nhere";
        let mut ci = chunks(input, &delim);
        match ci.next() {
            None => panic!("Expected a chunk but got none"),
            Some(Ok(c)) => assert_eq!(c, input),
            Some(e) => panic!("Got an unexpected error: {:?}", e)
        }
    }

    #[test]
    fn test_one_delimited_marks_end() {
        let delim = embl_delim();
        let input: &[u8] = b"some\nstuff\ngoes\nhere\n//\n";
        let mut ci = chunks(input, &delim);
        match ci.next() {
            None => panic!("Expected a chunk but got none"),
            Some(Ok(c)) => assert_eq!(c, input),
            Some(e) => panic!("Got an unexpected error: {:?}", e)
        }
    }

    #[test]
    fn test_two_delimited_marks_start() {
        let delim = fasta_delim();
        let input: &[u8] = b">seq 1\nagct\n>seq2\ngattaca\n";
        let seq1: &[u8] = b">seq 1\nagct\n";
        let seq2: &[u8] = b">seq2\ngattaca\n";

        let mut ci = chunks(input, &delim);
        match ci.next() {
            Some(Ok(c)) => assert_eq!(c, seq1),
            e => panic!("Expected seq 1 but got: {:?}", e)
        }
        match ci.next() {
            Some(Ok(c)) => assert_eq!(c, seq2),
            e => panic!("Expected seq 2 but got: {:?}", e)
        }
    }

    #[test]
    fn test_two_delimited_marks_end() {
        let delim = embl_delim();
        let input: &[u8] = b"seq 1\nagct\n//\nseq2\ngattaca\n//\n";
        let seq1: &[u8] = b"seq 1\nagct\n//\n";
        let seq2: &[u8] = b"seq2\ngattaca\n//\n";

        let mut ci = chunks(input, &delim);
        match ci.next() {
            Some(Ok(c)) => assert_eq!(c, seq1),
            e => panic!("Expected seq 1 but got: {:?}", e)
        }
        match ci.next() {
            Some(Ok(c)) => assert_eq!(c, seq2),
            e => panic!("Expected seq 2 but got: {:?}", e)
        }
    }
}