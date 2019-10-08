use nom::{
  IResult,
  branch::{
    alt,
  },
  bytes::complete::{
    tag,
  },
  character::complete::{ 
    digit1,
    alphanumeric1,
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
    map(parse_point, Local::Point),
    map(parse_between, Local::Between),
    parse_within,
    parse_span
  ))(input)
}


#[derive(Debug, PartialEq, Eq)]
enum Loc {
  Remote { within: String, at: Local },
  Local(Local)
}

fn parse_loc(input: &str) -> IResult<&str, Loc> {
  alt((
    map(
      tuple((alphanumeric1, tag(":"), parse_local)),
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

    parse_locOp("<345..500");
    parse_locOp("<1..888");
    parse_locOp("1..>888");
    parse_locOp("102.110");
    parse_locOp("123^124");
    parse_locOp("join(12..78,134..202)");
    parse_locOp("complement(34..126)");
    parse_locOp("complement(join(2691..4571,4918..5163))");
    parse_locOp("join(complement(4918..5163),complement(2691..4571))");
    parse_locOp("J00194.1:100..202");
    parse_locOp("join(1..100,J00194.1:100..202)");

  }
}