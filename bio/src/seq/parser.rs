use nom::{
    IResult,
    error::{ParseError},
    character::complete::{digit1},
    combinator::{map_res}};

pub trait Nommed<I, E> where Self: Sized {
  fn nom(input: I) -> IResult<I, Self, E>;
}

impl <'a, E : ParseError<&'a str>> Nommed<&'a str, E> for u32 
{
  fn nom(input: &'a str) -> IResult<&'a str, u32, E> {
    map_res(digit1, |d: &'a str| d.parse::<u32>())(input)
  }
}

// todo: implement stanza-based parsing
// todo: implement line-wrap parsing