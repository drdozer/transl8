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

#[derive(Debug)]
struct Stanza<'a> {
  /// The tanza tag 
  tag: Option<&'a str>,
  lines: Vec<&'a str>,
}


struct LeadingColumns {
  tag_columns: usize,
  merge_tags: bool,
}

struct LeadingColumnsIterator<'a, TV> {
  lcols: &'a LeadingColumns,
  tv_iterator: TV,
  next: Option<(Option<&'a str>, Option<&'a str>)>
}

impl <'a, TV> Iterator for LeadingColumnsIterator<'a, TV>
where TV : Iterator<Item=(Option<&'a str>, Option<&'a str>)> {
  type Item = Stanza<'a>;

  fn next(&mut self) -> Option<Self::Item> {
    match self.next {
      None => None,
      Some((tag, first_value)) => {
        let mut values: Vec<&str> = Vec::new();
        values.extend(first_value);
        loop {
          match self.tv_iterator.next() {
            None => {
              self.next = None;
              return Some(Stanza{
                tag,
                lines: values
              })
            },
            Some((t, v)) => {
              if t.is_none() || (self.lcols.merge_tags && t == tag) {
                values.extend(v);
                continue;
              } else {
                self.next = Some((t, v));
                return Some(Stanza{
                  tag,
                  lines: values
                })
              }
            }
          }
        }
      }
    }
  }
}

impl LeadingColumns {
  pub fn stanzas<'a, L>(&'a self, lines: L) -> impl Iterator<Item=Stanza<'a>>
  where L: Iterator<Item=&'a str>
  {
    let mut tag_values = lines.map(move |l| self.tag_value(l));
    let next = tag_values.next();

    LeadingColumnsIterator {
      lcols: self,
      tv_iterator: tag_values,
      next
    }
  }

  fn tag_value<'a>(&self, line: &'a str) -> (Option<&'a str>, Option<&'a str>) {
    if line.len() < self.tag_columns {
      (Some(line.trim()).filter(|t| !t.is_empty()), None)
    } else {
      let (t, v) = line.split_at(self.tag_columns);
      (
        Some(t.trim()).filter(|t| !t.is_empty()),
        Some(v.trim()).filter(|v| !v.is_empty())
      )
    }
  }
}

#[cfg(test)]
mod test {
  use super::*;

  #[test]
  fn test_stanzas_non_merging() {
    let non_merging = LeadingColumns { tag_columns: 5, merge_tags: false };

    let input = r#"
s1   this
     is
     data
s2   and
     more
s3   here
"#.trim().lines();

    let stanzas: Vec<Stanza<'_>> = non_merging.stanzas(input).collect();

    println!("Stanzas: {:?}", stanzas);
  }
}