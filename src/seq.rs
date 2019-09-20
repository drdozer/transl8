use std::io;

pub struct FastaRecord<'a> {
  pub descrLine: &'a str,
  pub seq: &'a str,
}

impl <'a> FastaRecord<'a> {

  pub fn write<W : io::Write>(&self, format: &FastaFormat, out: &mut W) -> Result<(), io::Error> {
    let ll = format.lineLength;
    let seq = self.seq;

    write!(out, ">")?;
    write!(out, "{}", self.descrLine)?;
    write!(out, "\n")?;

    let l = self.seq.len();
    let overhang = l % ll;

    for o in (0..l).step_by(ll) {
      let e = o + ll;
      let r = if e > l { l } else { e };
      write!(out, "{}", &seq[o..r])?;
      write!(out, "\n")?;
    }

    Ok({})
  }

}

pub struct FastaFormat {
  pub lineLength: usize,
}

impl FastaFormat {
  pub fn new() -> FastaFormat {
    FastaFormat { lineLength: 60 }
  }

  pub fn new_with_line_length(lineLength: usize) -> FastaFormat {
    FastaFormat { lineLength }
  }
}
