//! This crate has some usefully parsing structs and functions which can
//! be extended to multiple different uses. Some basic parsing functions are provided,
//! but the intention is to allow easy extension of both sources of data as well as
//! adding additional parsin functions.
//!
//! # Example
//!
//! ```
//! use hb_parse::StrParser;
//! use hb_parse::CommonParserFunctions;
//! let mut source = StrParser::new(
//!     "This is a word. And some \"Strings, amazing!\" 1 -2 12.3 (Or something like that) 2!",
//! );
//! assert_eq!(source.parse_word().unwrap(), "This".to_owned());
//! assert_eq!(source.parse_word().unwrap(), "is".to_owned());
//! assert_eq!(source.parse_word().unwrap(), "a".to_owned());
//! assert_eq!(source.parse_word().unwrap(), "word".to_owned());
//! assert_eq!(source.parse_symbol().unwrap(), '.');
//! source.consume_whitespace().ok();
//! assert_eq!(source.parse_word().unwrap(), "And".to_owned());
//! assert_eq!(source.parse_word().unwrap(), "some".to_owned());
//! assert_eq!(
//!     source.parse_string().unwrap(),
//!     "Strings, amazing!".to_owned()
//! );
//! assert_eq!(source.parse_num::<u32>().unwrap(), 1);
//! assert_eq!(source.parse_num::<i32>().unwrap(), -2);
//! assert_eq!(source.parse_num::<f32>().unwrap(), 12.3);
//! assert_eq!(
//!     source.parse_brackets().unwrap(),
//!     "Or something like that".to_owned()
//! );
//! assert_eq!(source.parse_num::<i64>().unwrap(), 2);
//! assert_eq!(source.parse_symbol().unwrap(), '!');
//! ```
//!
//! # Extending the functionality
//! It is possible to add additional sources of data that the parsing
//! functions can be used on by implementing the source::Source trait.
//!
//! It is also possible to add more parsing functions by creating new
//! traits and implementing using generics for Source structs.
//!
//! # Example adding new parsing function
//! In this example, a new parsing function is implemented for all Sources which
//! reads the next char and returns true if it is a 'T'.
//! ```
//! use hb_parse::StrParser;
//! use hb_parse::Source;
//! use hb_parse::ParseResult;
//! trait NewParseFuncs {
//!    fn new_func(&mut self) -> ParseResult<bool>;
//! }
//! impl <T: Source> NewParseFuncs for T {
//!    fn new_func(&mut self) -> ParseResult<bool> {
//!    // some logic for the parsing...
//!        match self.next() {
//!            Err(e) => return Err(e.make_inner().msg("could not do new_func").context(self.get_context())),
//!            Ok(None) => return Ok(false),
//!            Ok(Some((_,c))) => {
//!                if c == 'T' {
//!                    return Ok(true);
//!                } else {
//!                    return Ok(false);
//!                }
//!            }
//!        }
//!    }
//!}
//! let mut source = StrParser::new(
//!     "This is a word. And some \"Strings, amazing!\" 1 -2 12.3 (Or something like that) 2!",
//! );
//!assert_eq!(source.new_func().unwrap(), true);
//!assert_eq!(source.new_func().unwrap(), false);
//! ```
pub mod error;
pub mod parser_funcs;
pub mod source;
pub use self::parser_funcs::CommonParserFunctions;
pub use error::{ParseError, ParseResult, SourceEmpty, SourceError, SourceResult};
pub use hb_error::context;
pub use source::Source;

pub struct StrParser<'a> {
    s: &'a str,                                     // the raw source of chars
    sub_s: &'a str,      // the windowed str that the iter is created from
    window_start: usize, // the current start of the window of the str
    pointer: usize,      // the current location of the next char that will be provided
    iter: std::iter::Peekable<std::str::Chars<'a>>, //the iter used to extract chars
}

impl<'a> StrParser<'a> {
    pub fn new(s: &'a str) -> StrParser<'a> {
        StrParser {
            s,
            sub_s: s,
            window_start: 0,
            pointer: 0,
            iter: s.chars().peekable(),
        }
    }
}

impl Source for StrParser<'_> {
    fn next(&mut self) -> SourceResult<Option<(usize, char)>> {
        match self.iter.next() {
            Some(c) => {
                let ret = Ok(Some((self.pointer, c)));
                self.pointer += 1;
                ret
            }
            None => Ok(None),
        }
    }

    fn peek(&mut self) -> SourceResult<Option<(usize, char)>> {
        match self.iter.peek() {
            Some(c) => Ok(Some((self.pointer, *c))),
            None => Ok(None),
        }
    }

    fn move_back(&mut self, n: usize) -> SourceResult<()> {
        if self.pointer < n {
            return Err(SourceError::new().msg(format!(
                "attempted to move pointer ({}) back {} places past the start of the data",
                self.pointer, n
            )));
        }
        self.pointer -= n;
        self.iter = self.sub_s.chars().peekable();
        // consume so the pointer is at the next value
        if self.pointer != 0 {
            self.iter.nth(self.pointer - 1);
        }
        Ok(())
    }

    fn move_forward(&mut self, n: usize) -> SourceResult<()> {
        if self.pointer + n > self.sub_s.len() {
            return Err(SourceError::new().msg(format!(
                "attempted to move pointer ({}) forward {} places past the end of the data ({})",
                self.pointer,
                n,
                self.sub_s.len()
            )));
        }
        self.pointer += n;
        if n != 0 {
            // advance by n places -> nth(0) -> advances 1, n(1) advances 2...
            self.iter.nth(n - 1);
        }
        Ok(())
    }
    fn consume(&mut self, n: usize) -> SourceResult<()> {
        if n > self.sub_s.len() {
            return Err(SourceError::new().msg(format!(
                "attempted to consume {} chars when only {} remain",
                n,
                self.sub_s.len()
            )));
        }
        if n != 0 {
            // move window
            self.window_start += n;
            // move pointer
            if self.pointer > n {
                self.pointer -= n;
            } else {
                self.pointer = 0;
            }
            // recreate sub-string and iter
            self.sub_s = &self.s[self.window_start..];
            self.iter = self.sub_s.chars().peekable();
            // move iter to pointer
            if self.pointer != 0 {
                // advance by n places -> nth(0) -> advances 1, n(1) advances 2...
                self.iter.nth(self.pointer - 1);
            }
        }
        Ok(())
    }

    fn extract(&mut self, n: usize) -> SourceResult<String> {
        if n > self.sub_s.len() {
            return Err(SourceError::new().msg(format!(
                "attempted to extract {} chars when only {} remain",
                n,
                self.sub_s.len()
            )));
        }
        let ret = self.sub_s[0..n].to_string();
        if n != 0 {
            // move window
            self.window_start += n;
            // move pointer
            if self.pointer > n {
                self.pointer -= n;
            } else {
                self.pointer = 0;
            }
            // recreate sub-string and iter
            self.sub_s = &self.s[self.window_start..];
            self.iter = self.sub_s.chars().peekable();
            // move iter to pointer
            if self.pointer != 0 {
                // advance by n places -> nth(0) -> advances 1, n(1) advances 2...
                self.iter.nth(self.pointer - 1);
            }
        }
        Ok(ret)
    }
    fn read_substr(&mut self, start: usize, n: usize) -> SourceResult<String> {
        if start > self.sub_s.len() {
            return Err(SourceError::new().msg(format!(
                "attempted to read substring from start position {} when only {} remain",
                start,
                self.sub_s.len()
            )));
        }
        if start + n > self.sub_s.len() {
            return Err(SourceError::new().msg(format!(
                "attempted to read a substring of {} chars when only {} remain",
                n,
                self.sub_s.len() - start
            )));
        }
        if n == 0 {
            return Ok(String::new());
        }
        Ok(self.sub_s[start..start + n].to_string())
    }

    fn get_pointer_loc(&self) -> usize {
        self.pointer
    }

    fn reset_pointer_loc(&mut self) {
        self.pointer = 0;
        self.iter = self.sub_s.chars().peekable();
    }

    fn get_context(&self) -> String {
        let mut start_i = 0;
        if self.pointer > 40 {
            start_i = self.pointer - 40;
        }
        let mut end_i = start_i + 80;
        if self.sub_s.len() < end_i {
            end_i = self.sub_s.len();
        }
        format!(
            "{}\n{}{}\n",
            self.sub_s[start_i..end_i].to_owned(),
            " ".repeat(self.pointer - start_i),
            '^'
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn strsource_next_tests() {
        let mut source = StrParser::new("Something");
        assert_eq!(source.next().unwrap(), Some((0, 'S')));
        assert_eq!(source.next().unwrap(), Some((1, 'o')));
        assert_eq!(source.next().unwrap(), Some((2, 'm')));
        assert_eq!(source.next().unwrap(), Some((3, 'e')));
        assert_eq!(source.next().unwrap(), Some((4, 't')));
        assert_eq!(source.next().unwrap(), Some((5, 'h')));
        assert_eq!(source.next().unwrap(), Some((6, 'i')));
        assert_eq!(source.next().unwrap(), Some((7, 'n')));
        assert_eq!(source.next().unwrap(), Some((8, 'g')));
        assert_eq!(source.next().unwrap(), None);
    }
    #[test]
    fn strsource_peek_tests() {
        let mut source = StrParser::new("Something");
        assert_eq!(source.peek().unwrap(), Some((0, 'S')));
        assert_eq!(source.next().unwrap(), Some((0, 'S')));
        assert_eq!(source.peek().unwrap(), Some((1, 'o')));
        assert_eq!(source.next().unwrap(), Some((1, 'o')));
        assert_eq!(source.next().unwrap(), Some((2, 'm')));
        assert_eq!(source.next().unwrap(), Some((3, 'e')));
        assert_eq!(source.next().unwrap(), Some((4, 't')));
        assert_eq!(source.next().unwrap(), Some((5, 'h')));
        assert_eq!(source.next().unwrap(), Some((6, 'i')));
        assert_eq!(source.next().unwrap(), Some((7, 'n')));
        assert_eq!(source.next().unwrap(), Some((8, 'g')));
        assert_eq!(source.peek().unwrap(), None);
    }
    #[test]
    fn strsource_move_back_tests() {
        let mut source = StrParser::new("Something");
        assert_eq!(source.next().unwrap(), Some((0, 'S')));
        assert_eq!(source.next().unwrap(), Some((1, 'o')));
        assert_eq!(source.next().unwrap(), Some((2, 'm')));
        assert_eq!(source.move_back(2).unwrap(), ());
        assert_eq!(source.next().unwrap(), Some((1, 'o')));
        assert_eq!(source.next().unwrap(), Some((2, 'm')));
        assert_eq!(source.next().unwrap(), Some((3, 'e')));
        assert_eq!(source.next().unwrap(), Some((4, 't')));
        assert_eq!(source.next().unwrap(), Some((5, 'h')));
        assert_eq!(source.next().unwrap(), Some((6, 'i')));
        assert_eq!(source.next().unwrap(), Some((7, 'n')));
        assert_eq!(source.next().unwrap(), Some((8, 'g')));
        assert_eq!(source.next().unwrap(), None);
    }
    #[test]
    fn strsource_move_forward_tests() {
        let mut source = StrParser::new("Something");
        assert_eq!(source.next().unwrap(), Some((0, 'S')));
        assert_eq!(source.next().unwrap(), Some((1, 'o')));
        assert_eq!(source.next().unwrap(), Some((2, 'm')));
        assert_eq!(source.move_forward(2).unwrap(), ());
        assert_eq!(source.next().unwrap(), Some((5, 'h')));
        assert_eq!(source.next().unwrap(), Some((6, 'i')));
        assert_eq!(source.next().unwrap(), Some((7, 'n')));
        assert_eq!(source.next().unwrap(), Some((8, 'g')));
        assert_eq!(source.next().unwrap(), None);
    }
    #[test]
    fn strsource_consume_tests() {
        let mut source = StrParser::new("Something");
        assert_eq!(source.next().unwrap(), Some((0, 'S')));
        assert_eq!(source.next().unwrap(), Some((1, 'o')));
        assert_eq!(source.next().unwrap(), Some((2, 'm')));
        assert_eq!(source.consume(2).unwrap(), ());
        assert_eq!(source.next().unwrap(), Some((1, 'e')));
        assert_eq!(source.next().unwrap(), Some((2, 't')));
        assert_eq!(source.next().unwrap(), Some((3, 'h')));
        assert_eq!(source.next().unwrap(), Some((4, 'i')));
        assert_eq!(source.next().unwrap(), Some((5, 'n')));
        assert_eq!(source.next().unwrap(), Some((6, 'g')));
        assert_eq!(source.next().unwrap(), None);
    }
    #[test]
    fn strsource_extract_tests() {
        let mut source = StrParser::new("Something");
        assert_eq!(source.next().unwrap(), Some((0, 'S')));
        assert_eq!(source.next().unwrap(), Some((1, 'o')));
        assert_eq!(source.next().unwrap(), Some((2, 'm')));
        assert_eq!(source.extract(2).unwrap(), "So".to_string());
        assert_eq!(source.next().unwrap(), Some((1, 'e')));
        assert_eq!(source.next().unwrap(), Some((2, 't')));
        assert_eq!(source.next().unwrap(), Some((3, 'h')));
        assert_eq!(source.next().unwrap(), Some((4, 'i')));
        assert_eq!(source.next().unwrap(), Some((5, 'n')));
        assert_eq!(source.next().unwrap(), Some((6, 'g')));
        assert_eq!(source.next().unwrap(), None);
    }
    #[test]
    fn strsource_read_substr_tests() {
        let mut source = StrParser::new("Something");
        assert_eq!(source.next().unwrap(), Some((0, 'S')));
        assert_eq!(source.next().unwrap(), Some((1, 'o')));
        assert_eq!(source.next().unwrap(), Some((2, 'm')));
        assert_eq!(source.read_substr(0, 2).unwrap(), "So".to_string());
        assert_eq!(source.next().unwrap(), Some((3, 'e')));
        assert_eq!(source.next().unwrap(), Some((4, 't')));
        assert_eq!(source.read_substr(4, 2).unwrap(), "th".to_string());
    }
    #[test]
    fn strsource_get_pointer_loc_tests() {
        let mut source = StrParser::new("Something");
        assert_eq!(source.next().unwrap(), Some((0, 'S')));
        assert_eq!(source.next().unwrap(), Some((1, 'o')));
        assert_eq!(source.next().unwrap(), Some((2, 'm')));
        assert_eq!(source.get_pointer_loc(), 3);
        assert_eq!(source.next().unwrap(), Some((3, 'e')));
        assert_eq!(source.next().unwrap(), Some((4, 't')));
        assert_eq!(source.get_pointer_loc(), 5);
    }
    #[test]
    fn strsource_get_context_tests() {
        let mut source = StrParser::new(
            "This is a longer sentence, it has to be over 80 characters or my tests won't work...",
        );
        source.move_forward(10).ok();
        assert_eq!(source.get_context(), "This is a longer sentence, it has to be over 80 characters or my tests won't wor\n          ^\n".to_owned());
        source.move_forward(35).ok();

        assert_eq!(source.get_context(), "is a longer sentence, it has to be over 80 characters or my tests won't work...\n                                        ^\n".to_owned());
    }
}
