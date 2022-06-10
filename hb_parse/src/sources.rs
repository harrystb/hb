use crate::error::{ParseError, ParseResult};

/// Interface for different implementations of sources of data for the parser.
/// Provides various function which will allow the parsing of data without having
/// multiple buffers.
pub trait Source {
    /// Gets the next char and moves the pointer
    fn next(&mut self) -> ParseResult<Option<(usize, char)>>;
    /// Gets the next char but does not move the pointer
    fn peek(&mut self) -> ParseResult<Option<(usize, char)>>;
    /// Moves the pointer backwards
    fn move_back(&mut self, n: usize) -> ParseResult<()>;
    /// Moves the pointer forwards
    fn move_forward(&mut self, n: usize) -> ParseResult<()>;
    /// Consumes the chars at the begining moving the window
    /// forward without returning any of the chars
    fn consume(&mut self, n: usize) -> ParseResult<()>;
    /// Extracts the chars at the begining of the window
    /// moving the window forward and returning the chars
    fn extract(&mut self, n: usize) -> ParseResult<String>;
    /// Reads a sub-string from the window, without affecting
    /// any pointers
    fn read_substr(&mut self, start: usize, n: usize) -> ParseResult<String>;
    /// Get the current pointer location
    fn get_pointer_loc(&mut self) -> usize;
}

pub struct StrSource<'a> {
    s: &'a str,                                     // the raw source of chars
    sub_s: &'a str,      // the windowed str that the iter is created from
    window_start: usize, // the current start of the window of the str
    pointer: usize,      // the current location of the next char that will be provided
    iter: std::iter::Peekable<std::str::Chars<'a>>, //the iter used to extract chars
}

impl<'a> StrSource<'a> {
    pub fn new(s: &'a str) -> StrSource<'a> {
        StrSource {
            s: s,
            sub_s: s,
            window_start: 0,
            pointer: 0,
            iter: s.chars().peekable(),
        }
    }
}

impl Source for StrSource<'_> {
    fn next(&mut self) -> ParseResult<Option<(usize, char)>> {
        match self.iter.next() {
            Some(c) => {
                let ret = Ok(Some((self.pointer, c)));
                self.pointer += 1;
                ret
            }
            None => Ok(None),
        }
    }

    fn peek(&mut self) -> ParseResult<Option<(usize, char)>> {
        match self.iter.peek() {
            Some(c) => Ok(Some((self.pointer, *c))),
            None => Ok(None),
        }
    }

    fn move_back(&mut self, n: usize) -> ParseResult<()> {
        if self.pointer < n {
            return Err(ParseError::with_msg(format!(
                "attempted to move pointer ({}) back {} places past the start of the data",
                self.pointer, n
            )));
        }
        self.pointer = self.pointer - n;
        self.iter = self.sub_s.chars().peekable();
        // consume so the pointer is at the next value
        if self.pointer != 0 {
            self.iter.nth(self.pointer - 1);
        }
        Ok(())
    }

    fn move_forward(&mut self, n: usize) -> ParseResult<()> {
        if self.pointer + n > self.sub_s.len() {
            return Err(ParseError::with_msg(format!(
                "attempted to move pointer ({}) forward {} places past the end of the data ({})",
                self.pointer,
                n,
                self.sub_s.len()
            )));
        }
        self.pointer = self.pointer + n;
        if n != 0 {
            // advance by n places -> nth(0) -> advances 1, n(1) advances 2...
            self.iter.nth(n - 1);
        }
        Ok(())
    }
    fn consume(&mut self, n: usize) -> ParseResult<()> {
        if n > self.sub_s.len() {
            return Err(ParseError::with_msg(format!(
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
                self.pointer == 0;
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

    fn extract(&mut self, n: usize) -> ParseResult<String> {
        if n > self.sub_s.len() {
            return Err(ParseError::with_msg(format!(
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
                self.pointer == 0;
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
    fn read_substr(&mut self, start: usize, n: usize) -> ParseResult<String> {
        if start > self.sub_s.len() {
            return Err(ParseError::with_msg(format!(
                "attempted to read substring from start position {} when only {} remain",
                start,
                self.sub_s.len()
            )));
        }
        if start + n >= self.sub_s.len() {
            return Err(ParseError::with_msg(format!(
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

    fn get_pointer_loc(&mut self) -> usize {
        return self.pointer;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn strsource_next_tests() {
        let mut source = StrSource::new("Something");
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
        let mut source = StrSource::new("Something");
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
        let mut source = StrSource::new("Something");
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
        let mut source = StrSource::new("Something");
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
        let mut source = StrSource::new("Something");
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
        let mut source = StrSource::new("Something");
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
        let mut source = StrSource::new("Something");
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
        let mut source = StrSource::new("Something");
        assert_eq!(source.next().unwrap(), Some((0, 'S')));
        assert_eq!(source.next().unwrap(), Some((1, 'o')));
        assert_eq!(source.next().unwrap(), Some((2, 'm')));
        assert_eq!(source.get_pointer_loc(), 3);
        assert_eq!(source.next().unwrap(), Some((3, 'e')));
        assert_eq!(source.next().unwrap(), Some((4, 't')));
        assert_eq!(source.get_pointer_loc(), 5);
    }
}
