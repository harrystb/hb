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
pub mod checkers;
pub mod error;
pub mod sources;
pub use error::ParseError;

pub struct HbParser<'a> {
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
    pub fn parse(&self) -> Result<usize, ParseError> {}
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn parser_check_tests() {}
}
