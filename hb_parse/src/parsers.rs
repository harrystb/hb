use crate::error::{ParseError, ParseInnerError, ParseResult};
use crate::sources::Source;

pub trait CommonParserFunctions {
    /// Parses a word from the source.
    /// A word is a all alphanumberic characters leading up to a non-aplphanumeric character.
    fn parse_word(&mut self) -> ParseResult<String>;
    /// Parses a string from the source.
    /// A string is either a word, or a set of chars enclosed by " or '.
    fn parse_string(&mut self) -> ParseResult<String>;
    fn parse_brackets(&mut self) -> ParseResult<String>;
    fn parse_u64(&mut self) -> ParseResult<i64>;
    fn parse_i64(&mut self) -> ParseResult<i64>;
    fn parse_f64(&mut self) -> ParseResult<f64>;
    /// Checks if the next char matches the provided value.
    /// If it matches then the parser is moved forward.
    fn match_char(&mut self, val: char) -> ParseResult<bool>;
    /// Checks the upcoming chars if they match the int value provided.
    /// If it matches then the parser is moved forward.
    fn match_i64(&mut self, val: i64) -> ParseResult<()>;
    /// Checks the upcoming chars if they match the str value provided.
    /// If it matches then the parser is moved forward.
    fn match_str(&mut self, val: &str) -> ParseResult<()>;
}

impl<T: Source> CommonParserFunctions for T {
    fn parse_word(&mut self) -> ParseResult<String> {
        if self.get_pointer_loc() != 0 {
            return Err(ParseError::with_msg("Parser has already been used, and has left a pointer at position {} (which should be 0)."));
        }
        loop {
            match self.next() {
                Err(e) => {
                    return Err(ParseError::with_context(
                        ParseInnerError::Parse(Box::new(e)),
                        "could not parse word",
                    ))
                }
                Ok(None) => {
                    return Err(ParseError::with_msg(
                        "could not parse word as there are none left in the source",
                    ))
                }
                Ok(Some((i, c))) => {
                    if !c.is_alphanumeric() {
                        return Ok(self.extract(i)?);
                    }
                }
            }
        }
    }

    fn parse_string(&mut self) -> ParseResult<String> {
        if self.get_pointer_loc() != 0 {
            return Err(ParseError::with_msg("Parser has already been used, and has left a pointer at position {} (which should be 0)."));
        }
        let mut expected_ending = ' ';
        let mut first_char = 0;
        match self.next() {
            Err(e) => {
                return Err(ParseError::with_context(
                    ParseInnerError::Parse(Box::new(e)),
                    "could not parse string",
                ))
            }
            Ok(None) => {
                return Err(ParseError::with_msg(
                    "could not parse string as there are none left in the source",
                ))
            }
            Ok(Some((_, c))) => {
                if c == '\'' {
                    expected_ending = '\'';
                } else if c == '"' {
                    expected_ending = '"';
                } else {
                    match self.parse_word() {
                        Err(e) => {
                            return Err(ParseError::with_context(
                                ParseInnerError::Parse(Box::new(e)),
                                "could not parse string",
                            ))
                        }
                        Ok(s) => return Ok(s),
                    }
                }
            }
        }
        loop {
            match self.next() {
                Err(e) => {
                    return Err(ParseError::with_context(
                        ParseInnerError::Parse(Box::new(e)),
                        "could not parse string",
                    ))
                }
                Ok(None) => {
                    return Err(ParseError::with_msg(
                        "could not parse string as there are none left in the source",
                    ))
                }
                Ok(Some((i, c))) => {
                    if c == expected_ending {
                        if i == 1 {
                            return Ok(String::new());
                        }
                        let ret = Ok(self.read_substr(1, i - 1)?);
                        self.consume(i)?;
                        return ret;
                    }
                }
            }
        }
    }

    fn parse_brackets(&mut self) -> ParseResult<String> {
        if self.get_pointer_loc() != 0 {
            return Err(ParseError::with_msg("Parser has already been used, and has left a pointer at position {} (which should be 0)."));
        }
        let mut expected_ending = ' ';
        match self.next() {
            Err(e) => {
                return Err(ParseError::with_context(
                    ParseInnerError::Parse(Box::new(e)),
                    "could not parse brackets",
                ))
            }
            Ok(None) => {
                return Err(ParseError::with_msg(
                    "could not parse brackets as there are none left in the source",
                ))
            }
            Ok(Some((_, c))) => {
                if c == '(' {
                    expected_ending = ')';
                } else if c == '<' {
                    expected_ending = '>';
                } else if c == '[' {
                    expected_ending = ']';
                } else if c == '{' {
                    expected_ending = '}';
                } else {
                    return Err(ParseError::with_msg(format!(
                        "could not parse brackets as '{}' was found instead of a bracket",
                        c
                    )));
                }
            }
        }
        loop {
            match self.next() {
                Err(e) => {
                    return Err(ParseError::with_context(
                        ParseInnerError::Parse(Box::new(e)),
                        "could not parse brackets",
                    ))
                }
                Ok(None) => {
                    return Err(ParseError::with_msg(
                        "could not parse brackets as there are none left in the source",
                    ))
                }
                Ok(Some((i, c))) => {
                    if c == expected_ending {
                        let ret = Ok(self.read_substr(1, i - 1)?);
                        self.consume(i)?;
                        return ret;
                    }
                }
            }
        }
    }
    fn parse_u64(&mut self) -> ParseResult<i64> {}
    fn parse_i64(&mut self) -> ParseResult<i64> {}
    fn parse_f64(&mut self) -> ParseResult<f64> {}

    fn match_char(&mut self, val: char) -> ParseResult<bool> {
        if self.get_pointer_loc() != 0 {
            return Err(ParseError::with_msg("Parser has already been used, and has left a pointer at position {} (which should be 0)."));
        }
        match self.peek() {
            Err(e) => Err(ParseError::with_context(
                ParseInnerError::Parse(Box::new(e)),
                "could not parse char",
            )),
            Ok(None) => Err(ParseError::with_msg(
                "could not parse char as there are none left in the source",
            )),
            Ok(Some((_, c))) => {
                if c == val {
                    // remove the char from the source
                    self.consume(1)?;
                    return Ok(true);
                } else {
                    return Ok(false);
                }
            }
        }
    }

    fn match_str(&mut self, val: &str) -> ParseResult<()> {
        return Err(ParseError::with_msg("Not Implemented."));
    }
    fn match_i64(&mut self, val: i64) -> ParseResult<()> {
        return Err(ParseError::with_msg("Not Implemented."));
    }
}
