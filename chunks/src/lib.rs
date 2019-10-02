use std::io::{BufRead, Error, ErrorKind};

fn first_index_of(needle: &[u8], haystack: &[u8]) -> Option<usize> {
    for i in 0..=haystack.len() - needle.len() {
        if haystack[i..].iter().zip(needle).all(|(n, h)| n == h) {
            return Some(i);
        }
    }

    None
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

pub trait Chunkable {
    fn chunks_starting_with(self, sentinel: &[u8]) -> Chunker<'_, Self>
    where
        Self: BufRead + Sized,
    {
        Chunker {
            reader: self,
            sentinel,
            inclusive: false,
            buf: Vec::new(),
            from: 0,
            searched: 0,
            done: false,
        }
    }

    fn chunks_ending_with(self, sentinel: &[u8]) -> Chunker<'_, Self>
    where
        Self: BufRead + Sized,
    {
        Chunker {
            reader: self,
            sentinel,
            inclusive: true,
            buf: Vec::new(),
            from: 0,
            searched: 0,
            done: false,
        }
    }
}

pub struct Chunker<'a, R> {
    reader: R,
    sentinel: &'a [u8],
    inclusive: bool,
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
                    match extend_buffer(&mut self.buf, &mut self.reader) {
                        Ok(0) => {
                            self.done = true;
                            return None;
                        }
                        Ok(_) => continue,
                        Err(e) => return Some(Err(e)),
                    }
                } else {
                    match first_index_of(self.sentinel, &self.buf[self.searched..]) {
                        None => {
                            let cmp = compact_buf(self.from, &mut self.buf);

                            match extend_buffer(&mut self.buf, &mut self.reader) {
                                Ok(0) => {
                                    self.from = self.buf.len();
                                    self.done = true;
                                    return Some(Ok(self.buf[..].to_vec()));
                                }
                                Ok(_) => {
                                    self.from = 0;
                                    self.searched -= cmp;
                                    continue;
                                }
                                Err(e) => return Some(Err(e)),
                            }
                        }
                        Some(p) => {
                            let hit_end = p
                                + (if self.inclusive {
                                    self.sentinel.len()
                                } else {
                                    0
                                });
                            let hit = &self.buf[self.from..hit_end];
                            self.from = p;
                            self.searched = hit_end;
                            return Some(Ok(hit.to_vec()));
                        }
                    }
                }
            }
        }
    }
}
