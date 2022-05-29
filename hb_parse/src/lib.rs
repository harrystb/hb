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
//! assert_eq!(
//!     parser.parse(Box::new(&mut checkers::CharChecker::new('e', checkers::CheckerMode::UpToAndIncluding))),
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
    /// assert_eq!(
    ///     parser.parse(Box::new(&mut checkers::CharChecker::new('e', checkers::CheckerMode::UpToAndIncluding))),
    ///     Ok("te".to_owned())
    /// );
    /// ```
    pub fn parse(
        &mut self,
        checker: Box<&mut dyn checkers::ParseChecker>,
    ) -> Result<String, ParseError> {
        while let Some(c) = self.source.next() {
            match checker.parse(c) {
                Ok(Some(r)) => return Ok(r),
                Ok(None) => (),
                Err(e) => return Err(e),
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
