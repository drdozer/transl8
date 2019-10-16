//! # Feature Table
//!
//! Data model and parsers for the DDBJ/ENA/GenBank Feature Table.
//! 
//! See: http://www.insdc.org/files/feature_table.html

use nom::{
  IResult,
  branch::{
    alt,
  },
  bytes::complete::{
    tag,
    take_while_m_n,
    take_while,
    take_while1,
  },
  character::{
    is_alphanumeric,
  },
  combinator::{ 
    cut,
    map,
    opt,
    verify,
    },
  error::{
    ParseError,
  },
  multi::{
    // many1,
    separated_list,
  },
  sequence::{
    tuple,
  },
};

use super::parser::Nommed;


#[derive(Debug, PartialEq, Eq)]
pub struct FeatureTable {
  features: Vec<FeatureRecord>
}

#[derive(Debug, PartialEq, Eq)]
pub struct FeatureRecord {
  key: String,
  location: LocOp,
  qualifiers: Vec<Qualifier>
}

// impl <'a, E : ParseError<&'a str>> Nommed<&'a str, E> for FeatureRecord {
//   fn nom(input: &'a str) -> IResult<&'a str, FeatureRecord, E> {

//   }
// }



/// An ID that's valid within the feature table.
///
/// This is:
///   * At least one letter
///   * Upper case, lower case letters
///   * Numbers 0..9
///   * Underscore (_)
///   * Hyphen (-)
///   * Single quote (')
///   * Asterisk (*)
/// The maximum length is 20 characters.
#[derive(Debug, PartialEq, Eq)]
pub struct FtString(String);

// litle utility for ranges.
//
// Note: couldn't use 'a'..='b' because this is an iterator, so doesn't
// implement `Copy`.
#[derive(Clone, Copy)]
struct Interval<T>(T, T);
impl <T : PartialOrd> Interval<T> {
  fn contains(&self, e: &T) -> bool {
    self.0 <= *e &&
    *e <= self.1
  }
}

impl <'a, E : ParseError<&'a str>> Nommed<&'a str, E> for FtString {
  fn nom(input: &'a str) -> IResult<&'a str, FtString, E> {
  let uc = Interval('A', 'Z');
  let lc = Interval('a', 'z');
  let di = Interval('0', '9');
  let misc = "_-'*";

  let ft_char = {
    move |c: char| 
      uc.contains(&c) ||
      lc.contains(&c) ||
      di.contains(&c) ||
      misc.contains(c)
  };

  let alpha = {
    move |c: char|
      uc.contains(&c) ||
      lc.contains(&c)
  };
  
  map(
    verify(
      take_while_m_n(1, 20, ft_char),
      move |s: &str| s.chars().any(alpha)
    ),
    |s: &str| FtString(s.to_string())
  )(input)
}

}


#[derive(Debug, PartialEq, Eq)]
pub struct Qualifier {
  name: FtString,
  value: Option<QualifierValue>
}

impl <'a, E : ParseError<&'a str>> Nommed<&'a str, E> for Qualifier {
fn nom(input: &'a str) -> IResult<&'a str, Qualifier, E> {
  let parse_name = map(tuple((tag("/"), FtString::nom)), |(_, n)| n);

  let parse_value = map(tuple((tag("="), QualifierValue::nom)), |(_, v)| v);

  map(
    tuple((parse_name, opt(parse_value))),
    |(name, value)| Qualifier{ name, value }
  )(input)
}
}

#[derive(Debug, PartialEq, Eq)]
pub enum QualifierValue {
  QuotedText(String),
  VocabularyTerm(FtString),
  ReferenceNumber(u32),
}

impl <'a, E : ParseError<&'a str>> Nommed<&'a str, E> for QualifierValue{

fn nom(input: &'a str) -> IResult<&'a str, QualifierValue, E> {
  let parse_quoted_text =
    map(
      tuple((tag("\""), take_while(|c| c != '"'), tag("\""))),
      |(_, v, _): (&str, &str, &str)| QualifierValue::QuotedText(v.to_string()));

  let parse_vocabulary_term = 
    map(
      FtString::nom,
      QualifierValue::VocabularyTerm);

  let parse_reference_number = 
    map(
      tuple((tag("["), u32::nom, tag("]"))),
      |(_, d, _)| QualifierValue::ReferenceNumber(d));
  
  alt((
    parse_quoted_text,
    parse_vocabulary_term,
    parse_reference_number
  ))(input)
}
}

//
//
// Location data model starts here
//
// Should really be in a sub-module I guess
//
//


/// A point within a sequence, representing a specific nucleotide. Counts from 1.
#[derive(Debug, PartialEq, Eq)]
pub struct Point(u32);

impl <'a, E : ParseError<&'a str>> Nommed<&'a str, E> for Point {
  fn nom(input: &'a str) -> IResult<&'a str, Point, E> {
    map(u32::nom, Point)(input)
  }
}


/// A position between two bases in a sequence.
/// pub
/// For example, 122^123. The locations must be consecutive.
/// 
/// For example, 100^1 for a circular sequence of length 100.
#[derive(Debug, PartialEq, Eq)]
pub struct Between(u32, u32);

impl <'a, E : ParseError<&'a str>> Nommed<&'a str, E> for Between {
fn nom(input: &'a str) -> IResult<&'a str, Between, E> {
  map(
    tuple((
      u32::nom,
      tag("^"),
      u32::nom
    )),
    |(from, _, to)| Between(from, to)
  )(input)
}
}


#[derive(Debug, PartialEq, Eq)]
pub enum Position {
  Point(Point),
  Between(Between)
}

impl <'a, E : ParseError<&'a str>> Nommed<&'a str, E> for Position {
fn nom(input: &'a str) -> IResult<&'a str, Position, E> {
  alt((
    map(Between::nom, Position::Between),
    map(Point::nom, Position::Point)
  ))(input)
}
}


#[derive(Debug, PartialEq, Eq)]
pub enum Local {
  Point(Point),
  Between(Between),
  Within { from: Point, to: Point },
  Span { from: Position, to: Position, before_from: bool, after_to: bool },
}

impl Local {
  pub fn span(from: u32, to: u32) -> Local {
    Local::Span {
      from: Position::Point(Point(from)),
      to: Position::Point(Point(to)),
      before_from: false,
      after_to: false }
  }
}

impl <'a, E : ParseError<&'a str>> Nommed<&'a str, E> for Local {
  fn nom(input: &'a str) -> IResult<&'a str, Local, E> {
    let parse_within = map(
      tuple((Point::nom, tag("."), Point::nom)),
      |(from, _, to)| Local::Within { from, to });

    let parse_span = map(
      tuple((
        opt(tag("<")), Position::nom, tag(".."), opt(tag(">")), Position::nom)),
      |(before_from, from, _, after_to, to)| Local::Span {
        from,
        to,
        before_from: before_from.is_some(),
        after_to: after_to.is_some() }
    );

    alt((
      map(Between::nom, Local::Between),
      parse_within,
      parse_span,
      map(Point::nom, Local::Point), // must do this last as it's a prefix of the others
    ))(input)
  }
}


#[derive(Debug, PartialEq, Eq)]
pub enum Loc {
  Remote { within: String, at: Local },
  Local(Local)
}

impl <'a, E : ParseError<&'a str>> Nommed<&'a str, E> for Loc {
fn nom(input: &'a str) -> IResult<&'a str, Loc, E> {
  let parse_accession = take_while1(|c| {
    let b = c as u8;
    is_alphanumeric(b) || b == b'.'
  });

  alt((
    map(
      tuple((parse_accession, tag(":"), Local::nom)),
      |(within, _, at)| Loc::Remote { within: within.to_string(), at }
    ),
    map(Local::nom, Loc::Local)
  ))(input)
}
}



#[derive(Debug, PartialEq, Eq)]
pub enum LocOp {
  Loc(Loc),
  Complement(Box<LocOp>),
  Join(Vec<LocOp>),
  Order(Vec<LocOp>)
}

impl <'a, E : ParseError<&'a str>> Nommed<&'a str, E> for LocOp {
fn nom(input: &'a str) -> IResult<&'a str, LocOp, E> {
  
  let parse_complement = 
    map(
      tuple((
        tag("complement("),
        cut(LocOp::nom),
        tag(")")
      )),
      |(_, loc, _)| loc
    );

  let parse_join =
    map(
      tuple((
        tag("join("),
        cut(separated_list(tag(","), LocOp::nom)),
        tag(")")
      )),
      |(_, locs, _)| locs
    );

  let parse_order =
    map(
      tuple((
        tag("order("),
        cut(separated_list(tag(","), LocOp::nom)),
        tag(")")
      )),
      |(_, locs, _)| locs
    );

  alt((
    map(Loc::nom, LocOp::Loc),
    map(parse_complement, |loc| LocOp::Complement(Box::new(loc))),
    map(parse_join, LocOp::Join),
    map(parse_order, LocOp::Order)
  ))(input)
}
}

#[cfg(test)]
mod tests {

  use super::*;
  use nom::error::{
    convert_error,
    VerboseError,
  };

  fn assert_nom_to_expected<'a, T>() -> impl Fn(&'a str, T) -> ()
    where
      T: Nommed<&'a str, VerboseError<&'a str>> + std::fmt::Debug + PartialEq
  {
    move |input: &str, expected: T| {
      match T::nom(input) {
        Ok((rem, ref res)) if !rem.is_empty() => panic!("Non-empty remaining input {}, parsed out {:?}", rem, res),
        Ok((_, res)) => assert_eq!(res, expected, "Got result {:?} but expected {:?}", res, expected),
        Err(nom::Err::Error(e)) | Err(nom::Err::Failure(e)) => panic!("Problem: {}", convert_error(input, e)),
        e => panic!("Unknown error: {:?}", e)
      }
    }
  }

//   #[test]
//   fn test_parse_feature_record_from_spec() {

//     let expect = assert_nom_to_expected::<FeatureRecord>();

//     expect(
//       r#"
// source          1..1000
//                 /culture_collection="ATCC:11775"
//                 /culture_collection="CECT:515"
//       "#,
//       FeatureRecord {
//         key: "source".to_string(),
//         location: LocOp::Loc(Loc::Local(Local::span(1, 1000))),
//         qualifiers: vec![]
//       }
//     )

//   }

  #[test]
  fn test_parse_qualifiers_from_spec() {

    let expect = assert_nom_to_expected::<Qualifier>();

    expect(
      "/pseudo",
      Qualifier {
        name: FtString("pseudo".to_string()),
        value: None });
 
    expect(
      "/citation=[1]",
      Qualifier {
        name: FtString("citation".to_string()),
        value: Some(QualifierValue::ReferenceNumber(1)) });
 
    expect(
      "/gene=\"arsC\"",
      Qualifier {
        name: FtString("gene".to_string()),
        value: Some(QualifierValue::QuotedText("arsC".to_string()))});

    expect(
      "/rpt_type=DISPERSED",
      Qualifier {
        name: FtString("rpt_type".to_string()),
        value: Some(QualifierValue::VocabularyTerm(FtString("DISPERSED".to_string())))});
  }

  #[test]
  fn test_parse_locations_from_spec() {

    let expect = assert_nom_to_expected::<LocOp>();

    expect(
      "467",
      LocOp::Loc(Loc::Local(Local::Point(Point(467)))));

    expect(
      "340..565",
      LocOp::Loc(Loc::Local(Local::Span {
        from: Position::Point(Point(340)),
        to: Position::Point(Point(565)),
        before_from: false,
        after_to: false
        })));

    expect(
      "<345..500",
      LocOp::Loc(Loc::Local(Local::Span {
        from: Position::Point(Point(345)),
        to: Position::Point(Point(500)),
        before_from: true,
        after_to: false
      })));
    
    expect(
      "<1..888",
      LocOp::Loc(Loc::Local(Local::Span {
        from: Position::Point(Point(1)),
        to: Position::Point(Point(888)),
        before_from: true,
        after_to: false
      })));

    expect(
      "1..>888",
      LocOp::Loc(Loc::Local(Local::Span {
        from: Position::Point(Point(1)),
        to: Position::Point(Point(888)),
        before_from: false,
        after_to: true
      })));

    expect(
      "102.110",
      LocOp::Loc(Loc::Local(Local::Within { from: Point(102), to: Point(110) })));

    expect(
      "123^124",
      LocOp::Loc(Loc::Local(Local::Between(Between(123, 124)))));
    
    expect(
      "join(12..78)",
      LocOp::Join(vec![
        LocOp::Loc(Loc::Local(Local::span(12, 78)))]));
    
    expect(
      "join(12..78,134..202)",
      LocOp::Join(vec![
        LocOp::Loc(Loc::Local(Local::span(12, 78))),
        LocOp::Loc(Loc::Local(Local::span(134, 202)))]));

    expect(
      "complement(34..126)",
      LocOp::Complement(Box::new(LocOp::Loc(Loc::Local(Local::span(34, 126))))));
    
    expect(
      "complement(join(2691..4571,4918..5163))",
      LocOp::Complement(Box::new(LocOp::Join(vec![
        LocOp::Loc(Loc::Local(Local::span(2691, 4571))),
        LocOp::Loc(Loc::Local(Local::span(4918, 5163)))
      ]))));
    
    expect(
      "join(complement(4918..5163),complement(2691..4571))",
      LocOp::Join(vec![
        LocOp::Complement(Box::new(LocOp::Loc(Loc::Local(Local::span(4918, 5163))))),
        LocOp::Complement(Box::new(LocOp::Loc(Loc::Local(Local::span(2691, 4571)))))
      ]));

    expect(
      "J00194.1:100..202",
      LocOp::Loc(Loc::Remote{ within: String::from("J00194.1"), at: Local::span(100, 202) }));
    
    expect(
      "join(1..100,J00194.1:100..202)",
      LocOp::Join(vec![
        LocOp::Loc(Loc::Local(Local::span(1, 100))),
        LocOp::Loc(Loc::Remote { within: String::from("J00194.1"), at: Local::span(100, 202)})
      ]));

  }
}