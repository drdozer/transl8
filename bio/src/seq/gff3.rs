//! This is an implementaiton of the GFF3 spec.
//! 
//! https://github.com/The-Sequence-Ontology/Specifications/blob/master/gff3.md

// todo: This implementation doesn't implement any of the string excaping rules.
// todo: If and when it's extended to support that, consider using a wrapper
//   around string to enforce escaping
// todo: Handle comments

use std::fmt::Formatter;
use std::fmt::Display;
use std::collections::HashMap;
use std::str::FromStr;

// Fields use `String` rather than `&str` so that a record can live independently
// of a parse.
//
// Howeer, it may make sense to refactor this.
#[derive(Debug)]
pub struct GffRecord {
  pub seq_id: String,
  pub source: String,
  pub feature_type: String,
  pub start: OneBased,
  pub end: OneBased,
  pub score: Score,
  pub strand: Strand,
  pub phase: Phase,
  pub attributes: Attributes,
}


impl FromStr for GffRecord {
  type Err = GffParseError;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    let mut columns = s.trim().split("\t");

    let seq_id = columns.next()
      .ok_or(GffParseError::new(format!("No seqID column in {}", s)))
      .map(ToString::to_string)?;

    let source = columns.next()
      .ok_or(GffParseError::new(format!("No source columnin {}", s)))
      .map(ToString::to_string)?;

    let feature_type = columns.next()
      .ok_or(GffParseError::new(format!("No type column in {}", s)))
      .map(ToString::to_string)?;

    let start = columns.next()
      .ok_or(GffParseError::new(format!("No start column in {}", s)))
      .and_then(FromStr::from_str)?;

    let end = columns.next()
      .ok_or(GffParseError::new(format!("No seqID column in {}", s)))
      .and_then(FromStr::from_str)?;

    let score = columns.next()
      .ok_or(GffParseError::new(format!("No score column in {}", s)))
      .and_then(FromStr::from_str)?;

    let strand = columns.next()
      .ok_or(GffParseError::new(format!("No strand column in {}", s)))
      .and_then(FromStr::from_str)?;

    let phase = columns.next()
      .ok_or(GffParseError::new(format!("No phase column in {}", s)))
      .and_then(FromStr::from_str)?;

    let attributes = columns.next()
      .ok_or(GffParseError::new(format!("No attributes column in {}", s)))
      .and_then(FromStr::from_str)?;

    Ok(GffRecord { seq_id, source, feature_type, start, end, score, strand, phase, attributes })
  }
}


// Index counted from 1 rather than 0
#[derive(Debug, PartialEq)]
pub struct OneBased(u64);
impl OneBased {
  pub fn new(at: u64) -> OneBased { OneBased(at) }
  pub fn at(&self) -> u64 { self.0 }
}

impl FromStr for OneBased {
  type Err = GffParseError;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    u64::from_str(s).map(OneBased).map_err(|e| GffParseError::because(s, e))
  }
}

#[derive(Debug)]
pub struct Score(Option<f64>);

impl FromStr for Score {
  type Err = GffParseError;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s {
      "." => Ok(Score(None)),
      d => f64::from_str(d).map(|s| Score(Some(s))).map_err(|e| GffParseError::because(s, e))

    }
  }
}



#[derive(Debug)]
pub enum Strand {
  Positive,
  Negative,
  NoStrand,
  Unknown
}

impl FromStr for Strand {
  type Err = GffParseError;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s {
      "+" => Ok(Strand::Positive),
      "-" => Ok(Strand::Negative),
      "." => Ok(Strand::NoStrand),
      "?" => Ok(Strand::Unknown),
      e   => Err(GffParseError::new(format!("Cannot parse `{}` as a strand", e)))
    }
  }
}

// 0, 1, 2
#[derive(Debug)]
pub struct Phase(Option<u8>);

impl FromStr for Phase {
  type Err = GffParseError;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    if s == "." {
      Ok(Phase(None))
    } else {
      u8::from_str(s).map(|p| Phase(Some(p))).map_err(|e| GffParseError::because(s, e))
    }
  }
}



#[derive(Debug)]
pub struct Attributes(HashMap<String, String>);

impl FromStr for Attributes {
  type Err = GffParseError;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    let tvs: HashMap<String, String> = s.split(";").flat_map(|p| {
      let mut tv = p.split("=");
      match (tv.next(), tv.next()) {
        (Some(t), Some(v)) => Ok((t.to_string(), v.to_string())),
        _ => return Err(
          GffParseError::new(
            format!("Expected <tag>=<value> but got: {}", p))),
      }
    } ).collect();

    Ok(Attributes(tvs))
  }
}



#[derive(Debug)]
pub struct GffParseError(String);
impl GffParseError {
  pub fn new(msg: String) -> GffParseError { GffParseError(msg) }
  pub fn because<E : Display>(msg: &str, e: E) -> GffParseError { GffParseError(format!("in input `{}` because {}", msg, e)) } 
}
impl Display for GffParseError {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    write!(f, "Unable to parse GFF3 record because: {}", self.0)
  }
}