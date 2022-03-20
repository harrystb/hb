//! This crate has some usefully parsing structs and functions which can
//! be extended to multiple different uses.
//! # Example
//!
//! ```
//! use hb_parse::{HbParser, ParseEnding, ParseError};
//! let mut p = HbParser::new("tests");
//! let mut p = HbParser::new("tests");
//! assert_eq!(p.check(ParseEnding::AtChar('e')), Ok(1));
//! assert_eq!(
//!     p.check(ParseEnding::AtChar('z')),
//!     Err(ParseError::with_msg(
//!         "could not find 'z' in 'tests' when starting at 0"
//!     ))
//! );
//! ```
pub mod error;
pub use error::ParseError;
use std::collections::VecDeque;

pub struct HbParser<'a> {
    s: &'a str,
    loc: usize,
}

impl<'a> HbParser<'a> {
    pub fn new(s: &'a str) -> HbParser {
        HbParser { s: s, loc: 0 }
    }

    /// Checks in the string 's' for the ending from the location 'loc'
    ///
    /// # Example
    ///
    /// ```
    /// use hb_parse::{HbParser, ParseEnding, ParseError};
    /// let mut p = HbParser::new("tests");
    /// assert_eq!(p.check(ParseEnding::AtChar('e')), Ok(1));
    /// assert_eq!(
    ///     p.check(ParseEnding::AtChar('z')),
    ///     Err(ParseError::with_msg(
    ///         "could not find 'z' in 'tests' when starting at 0"
    ///     ))
    /// );
    /// ```
    pub fn check(&self, end: ParseEnding) -> Result<usize, ParseError> {
        use ParseEnding::*;
        match end {
            BeforeChar(end_char) => Ok(self.check_char(end_char)? - 1),
            AtChar(end_char) => Ok(self.check_char(end_char)?),
            BeforeStr(end_str) => Ok(self.check_str(end_str)? - 1),
            AtStr(end_str) => Ok(self.check_str(end_str)?),
            EndOfFile => Ok(self.s.len() - self.loc),
            EndOfLine => Ok(self.check_newline()?),
        }
    }
}

//Private Functions
impl<'a> HbParser<'a> {
    fn check_newline(&self) -> Result<usize, ParseError> {
        let mut iter = self.s.chars().enumerate();
        //consume loc -1 chars
        if self.loc > 0 {
            iter.nth(self.loc - 1);
        }
        let mut has_cr = false;
        for (i, c) in iter {
            match c {
                '\r' => has_cr = true,
                '\n' => {
                    if has_cr {
                        return Ok(i - self.loc - 1);
                    } else {
                        return Ok(i - self.loc);
                    }
                }
                _ => has_cr = false,
            }
        }
        Err(ParseError::with_msg(format!(
            "could not find a newline in '{}' when starting at {}",
            self.s, self.loc,
        )))
    }

    fn check_char(&self, end_char: char) -> Result<usize, ParseError> {
        let mut iter = self.s.chars().enumerate();
        //consume loc -1 chars
        if self.loc > 0 {
            iter.nth(self.loc - 1);
        }
        for (i, c) in iter {
            if c == end_char {
                return Ok(i - self.loc);
            }
        }
        Err(ParseError::with_msg(format!(
            "could not find '{}' in '{}' when starting at {}",
            end_char, self.s, self.loc,
        )))
    }

    fn check_str(&self, end_str: &str) -> Result<usize, ParseError> {
        let mut iter = self.s.chars().enumerate();
        //consume loc -1 chars
        if self.loc > 0 {
            iter.nth(self.loc - 1);
        }
        let mut buf: VecDeque<char> = VecDeque::with_capacity(end_str.len());
        for (i, c) in iter {
            buf.push_back(c);
            if *buf.back().unwrap() != end_str.chars().nth(buf.len() - 1).unwrap() {
                buf.clear();
            }
            if buf.len() == end_str.len() {
                return Ok(i - (end_str.len() - 1) - self.loc);
            }
        }
        Err(ParseError::with_msg(format!(
            "could not find '{}' in '{}' when starting at {}",
            end_str, self.s, self.loc,
        )))
    }
}

#[derive(Debug)]
pub enum ParseEnding<'a> {
    EndOfFile,
    EndOfLine,
    AtChar(char),
    BeforeChar(char),
    AtStr(&'a str),
    BeforeStr(&'a str),
}

trait Parser {
    fn parse(&mut self, c: char) -> Result<Option<(usize, usize)>, ParseError>;
}

struct CharParser {
    start_index: usize,
    chars_checked: usize,
    ending: char,
}

impl CharParser {
    fn new(ending: char, start_index: usize) -> CharParser {
        CharParser {
            start_index: start_index,
            chars_checked: 0,
            ending: ending,
        }
    }
}

impl<'a> Parser for CharParser {
    fn parse(&mut self, c: char) -> Result<Option<(usize, usize)>, ParseError> {
        if c == self.ending {
            return Ok(Some((
                self.start_index,
                self.start_index + self.chars_checked,
            )));
        } else {
            self.chars_checked += 1;
            return Ok(None);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn parser_check_tests() {
        let mut p = HbParser::new("tests");
        assert_eq!(p.check(ParseEnding::AtChar('e')), Ok(1));
        assert_eq!(p.check(ParseEnding::BeforeChar('e')), Ok(0));
        assert_eq!(p.check(ParseEnding::AtStr("st")), Ok(2));
        assert_eq!(p.check(ParseEnding::BeforeStr("st")), Ok(1));
        assert_eq!(p.check(ParseEnding::EndOfFile), Ok(5));
        assert_eq!(
            p.check(ParseEnding::AtChar('z')),
            Err(ParseError::with_msg(
                "could not find 'z' in 'tests' when starting at 0"
            ))
        );
        p = HbParser::new("tests\nwith newline");
        assert_eq!(p.check(ParseEnding::EndOfLine), Ok(5));
        p = HbParser::new("tests\r\nwith newline");
        assert_eq!(p.check(ParseEnding::EndOfLine), Ok(5));
        p = HbParser::new("te\rsts\r\nwith newline");
        assert_eq!(p.check(ParseEnding::EndOfLine), Ok(6));
    }
}
