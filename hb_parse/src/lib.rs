//! This crate has some usefully parsing structs and functions which can
//! be extended to multiple different uses.
//! # Example
//!
//! ```
//! use hb_parse::HbParser;
//! use hb_parse::sources;
//! use hb_parse::checkers;
//! let mut s = sources::StrSource::new("tests");
//! let mut parser = HbParser::new(&mut s);
//! let mut checkers : Vec<Box<dyn checkers::ParseChecker>> = vec![Box::new(checkers::CharChecker::new('e'))];
//! assert_eq!(
//!     parser.parse(&mut checkers),
//!     Ok("te".to_owned())
//! );
//! ```
pub mod checkers;
pub mod error;
pub mod sources;
pub use error::ParseError;

pub struct HbParser<'a> {
    source: &'a mut dyn sources::Source,
}

impl<'a> HbParser<'a> {
    pub fn new(source: &'a mut dyn sources::Source) -> HbParser {
        HbParser { source: source }
    }

    /// Checks in the string 's' for the ending from the location 'loc'
    ///
    /// # Example
    ///
    /// ```
    /// use hb_parse::HbParser;
    /// use hb_parse::sources;
    /// use hb_parse::checkers;
    /// let mut s = sources::StrSource::new("tests");
    /// let mut parser = HbParser::new(&mut s);
    /// let mut checkers : Vec<Box<dyn checkers::ParseChecker>> = vec![Box::new(checkers::CharChecker::new('e'))];
    /// assert_eq!(
    ///     parser.parse(&mut checkers),
    ///     Ok("te".to_owned())
    /// );
    /// ```
    pub fn parse(
        &mut self,
        checkers: &mut Vec<Box<dyn checkers::ParseChecker>>,
    ) -> Result<String, ParseError> {
        let mut buf = String::new();
        while let Some(c) = self.source.next() {
            buf.push(c);
            for checker in checkers.iter_mut() {
                if checker.parse(c) {
                    return Ok(buf);
                }
            }
        }
        Err(ParseError::with_msg(
            "ran out of chars without any checkers passing",
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn parser_check_tests() {}
}
