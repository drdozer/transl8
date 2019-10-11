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
pub struct Stanza<'a> {
  /// The tanza tag 
  pub tag: Option<&'a str>,
  pub lines: Vec<&'a str>,
}

/// Spec for stansas bases upon leading columns containing tags.
///
/// The first `tag_columns` characters contain the tag for the stanza.
/// All remaining columns make up the value(s) for that tag.
pub struct LeadingColumns {
  /// Number of columns to reserve for the tag.
  pub tag_columns: usize,
  /// When true, if successivel lines have the same tag, merge them into a
  /// single stanza.
  pub merge_tags: bool,
}

impl LeadingColumns {
  /// Convert an iterator over lines into an iterator over stanzas.
  /// 
  /// This takes ownership of the unerlying lines iterator.
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


/// Iterator to support converting an iterator over lines into an iterator over
/// stanzas. See [LeadingColumns::stanzas].
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
              // end of input - build and return a stanza, setting next to None
              self.next = None;
              return Some(Stanza{
                tag,
                lines: values
              })
            },
            Some((t, v)) => {
              if t.is_none() || (self.lcols.merge_tags && t == tag) {
                // value extending current values
                values.extend(v);
                continue;
              } else {
                // hit the start of the next stanza
                // - store it for next time, and return the stanza
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