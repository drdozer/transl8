use nom::{
  IResult,
  branch::{
    alt,
  },
  bytes::complete::{
    tag,
    take_while1,
  },
  character::{
    is_alphanumeric,
    complete::{ 
      digit1,
    }
  },
  combinator::{ 
    map,
    map_res,
    opt,
    },
  multi::{
    separated_list,
  },
  sequence::{
    tuple,
  },
};


// A record of an embl-like entry.
struct Embl {
  annotations: Vec<Annotation>,
  features: Vec<Feature>,
  sequence: String
}

struct Annotation {
  name: String,
  values: Vec<String>,
}

struct Feature {
  key: String,
  location: Loc,
  qualifiers: Vec<Annotation>,
}

// see http://www.insdc.org/files/feature_table.html


//
//
// Location data model starts here
//
// Should really be in a sub-module I guess
//
//

fn parse_u32(input: &str) -> IResult<&str, u32> {
  map_res(digit1, |d: &str| d.parse::<u32>())(input)
}


/// A point within a sequence, representing a specific nucleotide. Counts from 1.
#[derive(Debug, PartialEq, Eq)]
struct Point(u32);

fn parse_point(input: &str) -> IResult<&str, Point> {
  map(parse_u32, Point)(input)
}


/// A position between two bases in a sequence.
/// 
/// For example, 122^123. The locations must be consecutive.
/// 
/// For example, 100^1 for a circular sequence of length 100.
#[derive(Debug, PartialEq, Eq)]
struct Between(u32, u32);

fn parse_between(input: &str) -> IResult<&str, Between> {
  map(
    tuple((
      parse_u32,
      tag("^"),
      parse_u32
    )),
    |(from, _, to)| Between(from, to)
  )(input)
}

#[derive(Debug, PartialEq, Eq)]
enum Position {
  Point(Point),
  Between(Between)
}

fn parse_position(input: &str) -> IResult<&str, Position> {
  alt((
    map(parse_between, Position::Between),
    map(parse_point, Position::Point)
  ))(input)
}


#[derive(Debug, PartialEq, Eq)]
enum Local {
  Point(Point),
  Between(Between),
  Within { from: Point, to: Point },
  Span { from: Position, to: Position, before_from: bool, after_to: bool },
}

impl Local {
  fn span(from: u32, to: u32) -> Local {
    Local::Span {
      from: Position::Point(Point(from)),
      to: Position::Point(Point(to)),
      before_from: false,
      after_to: false }
  }
}

fn parse_local(input: &str) -> IResult<&str, Local> {
  let parse_within = map(
    tuple((parse_point, tag("."), parse_point)),
    |(from, _, to)| Local::Within { from, to });

  let parse_span = map(
    tuple((
      opt(tag("<")), parse_position, tag(".."), opt(tag(">")), parse_position)),
    |(before_from, from, _, after_to, to)| Local::Span {
      from,
      to,
      before_from: before_from.is_some(),
      after_to: after_to.is_some() }
  );

  alt((
    map(parse_between, Local::Between),
    parse_within,
    parse_span,
    map(parse_point, Local::Point), // must do this last as it's a prefix of the others
  ))(input)
}


#[derive(Debug, PartialEq, Eq)]
enum Loc {
  Remote { within: String, at: Local },
  Local(Local)
}

fn parse_loc(input: &str) -> IResult<&str, Loc> {
  let parse_accession = take_while1(|c| {
    let b = c as u8;
    is_alphanumeric(b) || b == b'.'
  });

  alt((
    map(
      tuple((parse_accession, tag(":"), parse_local)),
      |(within, _, at)| Loc::Remote { within: within.to_string(), at }
    ),
    map(parse_local, Loc::Local)
  ))(input)
}



#[derive(Debug, PartialEq, Eq)]
enum LocOp {
  Loc(Loc),
  Complement(Box<LocOp>),
  Join(Vec<LocOp>),
  Order(Vec<LocOp>)
}

fn parse_locOp(input: &str) -> IResult<&str, LocOp> {
  let parse_locOps = |i| separated_list(tag(","), parse_locOp)(i);

  let parse_complement = 
    map(
      tuple((
        tag("complement("),
        parse_locOp,
        tag(")")
      )),
      |(_, loc, _)| loc
    );

  let parse_join =
    map(
      tuple((
        tag("join("),
        parse_locOps,
        tag(")")
      )),
      |(_, locs, _)| locs
    );

  let parse_order =
    map(
      tuple((
        tag("order("),
        parse_locOps,
        tag(")")
      )),
      |(_, locs, _)| locs
    );

  alt((
    map(parse_loc, LocOp::Loc),
    map(parse_complement, |loc| LocOp::Complement(Box::new(loc))),
    map(parse_join, LocOp::Join),
    map(parse_order, LocOp::Order)
  ))(input)
}

#[cfg(test)]
mod tests {

  use super::*;

  #[test]
  fn test_parse_locations_from_spec() {
    let parse_locOp = |input| {
      match super::parse_locOp(input) {
        Ok((rem, ref res)) if !rem.is_empty() => panic!("Non-empty remaining input {}, parsed out {:?}", rem, res),
        Ok((_, res)) => res,
        e => panic!("Problem parsing: {:?}", e)
      }
    };

    assert_eq!(
      parse_locOp("467"),
      LocOp::Loc(Loc::Local(Local::Point(Point(467)))));

    assert_eq!(
      parse_locOp("340..565"),
      LocOp::Loc(Loc::Local(Local::Span {
        from: Position::Point(Point(340)),
        to: Position::Point(Point(565)),
        before_from: false,
        after_to: false
        })));

    assert_eq!(
      parse_locOp("<345..500"),
      LocOp::Loc(Loc::Local(Local::Span {
        from: Position::Point(Point(345)),
        to: Position::Point(Point(500)),
        before_from: true,
        after_to: false
      })));
    
    assert_eq!(
      parse_locOp("<1..888"),
      LocOp::Loc(Loc::Local(Local::Span {
        from: Position::Point(Point(1)),
        to: Position::Point(Point(888)),
        before_from: true,
        after_to: false
      })));

    
    assert_eq!(
      parse_locOp("1..>888"),
      LocOp::Loc(Loc::Local(Local::Span {
        from: Position::Point(Point(1)),
        to: Position::Point(Point(888)),
        before_from: false,
        after_to: true
      })));

    assert_eq!(
      parse_locOp("102.110"),
      LocOp::Loc(Loc::Local(Local::Within { from: Point(102), to: Point(110) })));

    assert_eq!(
      parse_locOp("123^124"),
      LocOp::Loc(Loc::Local(Local::Between(Between(123, 124)))));
    
    assert_eq!(
      parse_locOp("join(12..78,134..202)"),
      LocOp::Join(vec![
        LocOp::Loc(Loc::Local(Local::span(12, 78))),
        LocOp::Loc(Loc::Local(Local::span(134, 202)))]));

    assert_eq!(
      parse_locOp("complement(34..126)"),
      LocOp::Complement(Box::new(LocOp::Loc(Loc::Local(Local::span(34, 126))))));
    
    assert_eq!(
      parse_locOp("complement(join(2691..4571,4918..5163))"),
      LocOp::Complement(Box::new(LocOp::Join(vec![
        LocOp::Loc(Loc::Local(Local::span(2691, 4571))),
        LocOp::Loc(Loc::Local(Local::span(4918, 5163)))
      ]))));
    
    assert_eq!(
      parse_locOp("join(complement(4918..5163),complement(2691..4571))"),
      LocOp::Join(vec![
        LocOp::Complement(Box::new(LocOp::Loc(Loc::Local(Local::span(4918, 5163))))),
        LocOp::Complement(Box::new(LocOp::Loc(Loc::Local(Local::span(2691, 4571)))))
      ]));

    assert_eq!(
      parse_locOp("J00194.1:100..202"),
      LocOp::Loc(Loc::Remote{ within: String::from("J00194.1"), at: Local::span(100, 202) }));
    
    assert_eq!(
      parse_locOp("join(1..100,J00194.1:100..202)"),
      LocOp::Join(vec![
        LocOp::Loc(Loc::Local(Local::span(1, 100))),
        LocOp::Loc(Loc::Remote { within: String::from("J00194.1"), at: Local::span(100, 202)})
      ]));

  }
}