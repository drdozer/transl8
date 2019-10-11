use std::io;

extern crate nom;
use nom::{
  IResult,
  branch::{
    alt,
  },
  bytes::complete::{
    tag,
    take_while,
  },
  character::complete::{
    char,
    line_ending,
    space0,
    space1,
    },
  combinator::{
    map,
    cut,
    opt,
    },
  error::{
    context,
  },
  multi::{
    many0,
    separated_list,
  },
  sequence::{
    preceded,
    tuple,
    terminated
    },
};

#[derive(Debug, PartialEq)]
pub struct FastaDescription {
  pub identifier: Option<String>,
  pub description: Option<String>
}

impl FastaDescription {
  pub fn read(txt: &str) -> Self {
    let mut parts = txt.splitn(2, ' ');
    let id = parts.next();
    let de = parts.next();

    FastaDescription {
      identifier: id.map(|x| x.to_string()),
      description: de.map(|x| x.to_string())
    }
  }
}

#[derive(Debug, PartialEq)]
pub struct FastaRecord {
  pub descr_line: String,
  pub seq: String,
}

impl FastaRecord {
  pub fn descr_line(id: Option<&str>, descr: Option<&str>) -> String {
    let mut dl = String::new();
    for id in id { dl.push_str(id) }
    for de in descr { dl.push(' '); dl.push_str(de) }

    dl
  }
  
  pub fn write<W : io::Write>(&self, format: &FastaFormat, out: &mut W) -> Result<(), io::Error> {
    let ll = format.line_length;

    write!(out, ">")?;
    writeln!(out, "{}", self.descr_line)?;

    let l = self.seq.len();
    //let overhang = l % ll;

    for o in (0..l).step_by(ll) {
      let e = o + ll;
      let r = if e > l { l } else { e };
      writeln!(out, "{}", &self.seq[o..r])?;
    }

    Ok(())
  }

}


fn until_line_end(input: &str) -> IResult<&str, &str> {
  let eol_chars = "\r\n";
  take_while(move |c| !eol_chars.contains(c))(input)
}

fn line_end(input: &str) -> IResult<&str, &str> {
  alt((tag("\n"), tag("\r\n")))(input)
}

fn parse_fasta_header(input: &str) -> IResult<&str, String> {
  context(
    "description_line",
    map(
      tuple((
        space0,
        char('>'),
        cut(until_line_end),
        line_end
      )),
      |(_, _, descr, _)| {
        descr.to_string()
      }
    )
  )(input)
}

fn parse_seq_line(input: &str) -> IResult<&str, Vec<&str>> {
  context(
    "sequence_line",
    preceded(
      space0,
      terminated(
        separated_list(
          space1,
          take_while(|c| !(c == '>' || c == ' ' || c == '\t' || c == '\r' || c == '\n'))
        ),
        opt(line_ending)
      )
    )
  )(input)
}

fn parse_seq_lines(input: &str) -> IResult<&str, Vec<Vec<&str>>> {
  many0(parse_seq_line)(input)
}

pub fn parse_fasta(input: &str) -> IResult<&str, FastaRecord> {
  context(
    "fasta_record",
    map(
      tuple((
        parse_fasta_header,
        parse_seq_lines)),
      |(dl, sls)| FastaRecord {
        descr_line: dl,
        seq: {
          let mut s = String::new();
          for sl in sls {
            s.extend(sl)
          }
          s
        }
      }
    )
  )(input)
}

pub fn parse_fastas(input: &str) -> IResult<&str, Vec<FastaRecord>> {
  many0(parse_fasta)(input)
}

#[derive(Default)]
pub struct FastaFormat {
  pub line_length: usize,
}

impl FastaFormat {
  pub fn new() -> FastaFormat {
    FastaFormat { line_length: 60 }
  }

  pub fn _new_with_line_length(line_length: usize) -> FastaFormat {
    FastaFormat { line_length }
  }
}



#[cfg(test)]
mod tests {

  use super::*;

  #[test]
  fn test_until_line_end() {
    let input = "id descr\n";
    let output = "id descr";
    let rem = "\n";
    assert_eq!(until_line_end(input), Ok((rem, output)));
  }

  #[test]
  fn test_line_end() {
    let input = "\n";
    let output = "\n";
    let rem = "";
    assert_eq!(line_end(input), Ok((rem, output)));
  }

  #[test]
  fn test_terminated() {
    let input = "id descr\n";
    let output = "id descr";
    let rem = "";
    assert_eq!(terminated(until_line_end, line_end)(input), Ok((rem, output)));
  }

  #[test]
  fn test_parse_fasta_header() {
    let input = ">id descr\n";
    let output = "id descr".to_string();
    let rem = "";
    assert_eq!(parse_fasta_header(input), Ok((rem, output)));
  }

  #[test]
  fn test_parse_seq_line_with_line_end() {
    let input = "atgcatgcgtcgtatcgta\n";
    let output = vec!["atgcatgcgtcgtatcgta"];
    let rem = "";
    assert_eq!(parse_seq_line(input), Ok((rem, output)));
  }

  #[test]
  fn test_parse_seq_line_without_line_end() {
    let input = "atgcatgcgtcgtatcgta";
    let output = vec!["atgcatgcgtcgtatcgta"];
    let rem = "";
    assert_eq!(parse_seq_line(input), Ok((rem, output)));
  }

  #[test]
  fn test_parse_seq_lines_end() {
    let input = "atgcatgcgtcgtatcgta\ngcgtcgatctgca\n";
    let output = vec![vec!["atgcatgcgtcgtatcgta"], vec!["gcgtcgatctgca"]];
    let rem = "";
    assert_eq!(parse_seq_lines(input), Ok((rem, output)));
  }

  #[test]
  fn test_parse_seq_lines_next_record() {
    let input = "atgcatgcgtcgtatcgta\ngcgtcgatctgca\n>";
    let output = vec![vec!["atgcatgcgtcgtatcgta"], vec!["gcgtcgatctgca"]];
    let rem = ">";
    assert_eq!(parse_seq_lines(input), Ok((rem, output)));
  }

  #[test]
  fn test_parse_fasta_record() {
    let input = ">id descr\natgcatgcgtcgtatcgta\ngcgtcgatctgca\n>";
    let output = FastaRecord {
      descr_line: "id descr".to_string(),
      seq: "atgcatgcgtcgtatcgtagcgtcgatctgca".to_string()
    };
    let rem = ">";
    assert_eq!(parse_fasta(input), Ok((rem, output)));
  }

  #[test]
  fn test_parse_fasta_record_1() {
    let input = ">id descr\natgcatgcgtcgtatcgta\ngcgtcgatctgca\n";
    let output = FastaRecord {
      descr_line: "id descr".to_string(),
      seq: "atgcatgcgtcgtatcgtagcgtcgatctgca".to_string()
    };
    let rem = "";
    assert_eq!(parse_fastas(input), Ok((rem, vec![output])));
  }

  // #[test]
  // fn test_parse_fasta_records() {
  //   let input = ">id descr\natgcatgcgtcgtatcgta\ngcgtcgatctgca\n>id descr\natgcatgcgtcgtatcgta\ngcgtcgatctgca\n";
  //   let output = FastaRecord {
  //     descr_line: "id descr".to_string(),
  //     seq: "atgcatgcgtcgtatcgtagcgtcgatctgca".to_string()
  //   };
  //   let rem = "";
  //   assert_eq!(parse_fastas(input), Ok((rem, output)));
  // }
}