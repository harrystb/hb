use crate::error::{ParseError, ParseInnerError, ParseResult};
use crate::source::Source;
use std::fmt::Display;
use std::ops::{Add, Mul, Rem, Sub};
use std::str::FromStr;

pub trait CommonParserFunctions {
    /// Parses a word from the source.
    /// A word is a all alphanumberic characters leading up to a non-aplphanumeric character.
    fn parse_word(&mut self) -> ParseResult<String>;
    /// Parses a string from the source.
    /// A string is either a word, or a set of chars enclosed by " or '.
    fn parse_string(&mut self) -> ParseResult<String>;
    /// Parses the string until the other end of the brackets is found.
    fn parse_brackets(&mut self) -> ParseResult<String>;
    /// Parses a number (eg i32, i64, u32, u64, f32, f64)
    fn parse_num<N: FromStr + PartialEq + PartialOrd + Add<N> + Sub<N> + Mul<N> + Rem<N>>(
        &mut self,
    ) -> ParseResult<N>;
    /// Parses a symbol which is defined as non-alphanumeric and non-whitespace.
    fn parse_symbol(&mut self) -> ParseResult<char>;
    /// Checks if the next char matches the provided value.
    /// If it matches then the parser is moved forward.
    fn match_char(&mut self, val: char) -> ParseResult<bool>;
    /// Checks the upcoming chars if they match the int value provided.
    /// If it matches then the parser is moved forward.
    fn match_num<N: Display + PartialEq + PartialOrd + Add<N> + Sub<N> + Mul<N> + Rem<N>>(
        &mut self,
        val: N,
    ) -> ParseResult<bool>;
    /// Checks the upcoming chars if they match the str value provided.
    /// If it matches then the parser is moved forward.
    fn match_str(&mut self, val: &str) -> ParseResult<bool>;
    fn consume_whitespace(&mut self) -> ParseResult<()>;
}

impl<T: Source> CommonParserFunctions for T {
    fn parse_word(&mut self) -> ParseResult<String> {
        if self.get_pointer_loc() != 0 {
            return Err(ParseError::with_msg(format!("Parser has already been used, and has left a pointer at position {} (which should be 0).", self.get_pointer_loc())));
        }
        loop {
            match self.next() {
                Err(e) => {
                    self.reset_pointer_loc();
                    return Err(ParseError::with_context(
                        ParseInnerError::Parse(Box::new(e)),
                        "could not parse word",
                    ));
                }
                Ok(None) => {
                    self.reset_pointer_loc();
                    return Err(ParseError::with_msg(
                        "could not parse word as there are none left in the source",
                    ));
                }
                Ok(Some((i, c))) => {
                    if !c.is_alphanumeric() {
                        let r = self.read_substr(0, i)?;
                        if c.is_whitespace() {
                            self.consume(i + 1)?;
                        } else {
                            self.consume(i)?;
                            self.reset_pointer_loc();
                        }
                        return Ok(r);
                    }
                }
            }
        }
    }

    fn parse_string(&mut self) -> ParseResult<String> {
        if self.get_pointer_loc() != 0 {
            return Err(ParseError::with_msg(format!("Parser has already been used, and has left a pointer at position {} (which should be 0).", self.get_pointer_loc())));
        }
        let mut expected_ending = ' ';
        let mut first_char = 0;
        match self.next() {
            Err(e) => {
                self.reset_pointer_loc();
                return Err(ParseError::with_context(
                    ParseInnerError::Parse(Box::new(e)),
                    "could not parse string",
                ));
            }
            Ok(None) => {
                self.reset_pointer_loc();
                return Err(ParseError::with_msg(
                    "could not parse string as there are none left in the source",
                ));
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
                    self.reset_pointer_loc();
                    return Err(ParseError::with_context(
                        ParseInnerError::Parse(Box::new(e)),
                        "could not parse string",
                    ));
                }
                Ok(None) => {
                    self.reset_pointer_loc();
                    return Err(ParseError::with_msg(
                        "could not parse string as there are none left in the source",
                    ));
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
            return Err(ParseError::with_msg(format!("Parser has already been used, and has left a pointer at position {} (which should be 0).", self.get_pointer_loc())));
        }
        let mut expected_ending = ' ';
        let mut level = 1;
        match self.next() {
            Err(e) => {
                self.reset_pointer_loc();
                return Err(ParseError::with_context(
                    ParseInnerError::Parse(Box::new(e)),
                    "could not parse brackets",
                ));
            }
            Ok(None) => {
                self.reset_pointer_loc();
                return Err(ParseError::with_msg(
                    "could not parse brackets as there are none left in the source",
                ));
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
                    self.reset_pointer_loc();
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
                    self.reset_pointer_loc();
                    return Err(ParseError::with_context(
                        ParseInnerError::Parse(Box::new(e)),
                        "could not parse brackets",
                    ));
                }
                Ok(None) => {
                    self.reset_pointer_loc();
                    return Err(ParseError::with_msg(
                        "could not parse brackets as there are none left in the source",
                    ));
                }
                Ok(Some((i, c))) => {
                    if c == expected_ending {
                        level -= 1;
                        if level == 0 {
                            let ret = Ok(self.read_substr(1, i - 1)?);
                            self.consume(i)?;
                            return ret;
                        }
                    }
                }
            }
        }
    }

    fn parse_num<N: FromStr + PartialEq + PartialOrd + Add<N> + Sub<N> + Mul<N> + Rem<N>>(
        &mut self,
    ) -> ParseResult<N> {
        if self.get_pointer_loc() != 0 {
            return Err(ParseError::with_msg(format!("Parser has already been used, and has left a pointer at position {} (which should be 0).", self.get_pointer_loc())));
        }
        loop {
            match self.next() {
                Err(e) => {
                    self.reset_pointer_loc();
                    return Err(ParseError::with_context(
                        ParseInnerError::Parse(Box::new(e)),
                        "could not parse num",
                    ));
                }
                Ok(None) => {
                    self.reset_pointer_loc();
                    return Err(ParseError::with_msg(
                        "could not parse num as there are none left in the source",
                    ));
                }
                Ok(Some((i, c))) => {
                    if !c.is_ascii_digit() {
                        if i == 1 {
                            // if the first char is a - then it would be a negative number
                            if c == '-' || c == '+' {
                                continue;
                            }
                            self.reset_pointer_loc();
                            return Err(ParseError::with_msg(format!(
                                "could not parse num as there next char '{}' is not a digit",
                                c,
                            )));
                        }
                        let substr = self.extract(i).unwrap();
                        match substr.parse::<N>() {
                            Err(_) => {
                                self.reset_pointer_loc();
                                return Err(ParseError::with_msg(format!(
                                    "could not parse num from str '{}'",
                                    substr
                                )));
                            }
                            Ok(n) => return Ok(n),
                        }
                    }
                }
            }
        }
    }

    fn parse_symbol(&mut self) -> ParseResult<char> {
        if self.get_pointer_loc() != 0 {
            return Err(ParseError::with_msg(format!("Parser has already been used, and has left a pointer at position {} (which should be 0).", self.get_pointer_loc())));
        }
        match self.peek() {
            Err(e) => Err(ParseError::with_context(
                ParseInnerError::Parse(Box::new(e)),
                "could not parse symbol",
            )),
            Ok(None) => Err(ParseError::with_msg(
                "could not parse symbol as there are none left in the source",
            )),
            Ok(Some((_, c))) => {
                if !c.is_whitespace() && !c.is_ascii_alphanumeric() {
                    // remove the char from the source
                    self.consume(1)?;
                    return Ok(c);
                } else {
                    return Err(ParseError::with_msg(format!(
                        "cound not parse symbol because '{}' is not classified as a symbol",
                        c
                    )));
                }
            }
        }
    }

    fn match_char(&mut self, val: char) -> ParseResult<bool> {
        if self.get_pointer_loc() != 0 {
            return Err(ParseError::with_msg(format!("Parser has already been used, and has left a pointer at position {} (which should be 0).", self.get_pointer_loc())));
        }
        match self.peek() {
            Err(e) => Err(ParseError::with_context(
                ParseInnerError::Parse(Box::new(e)),
                "could not match char",
            )),
            Ok(None) => Err(ParseError::with_msg(
                "could not match char as there are none left in the source",
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

    fn match_str(&mut self, val: &str) -> ParseResult<bool> {
        if self.get_pointer_loc() != 0 {
            return Err(ParseError::with_msg(format!("Parser has already been used, and has left a pointer at position {} (which should be 0).", self.get_pointer_loc())));
        }
        let mut match_iter = val.chars();
        let mut next_char = match match_iter.next() {
            Some(c) => c,
            None => {
                return Err(ParseError::with_msg(
                    "cannot match str as the str provided is empty",
                ))
            }
        };
        loop {
            match self.next() {
                Err(e) => {
                    self.reset_pointer_loc();
                    return Err(ParseError::with_context(
                        ParseInnerError::Parse(Box::new(e)),
                        "could not match str",
                    ));
                }
                Ok(None) => {
                    self.reset_pointer_loc();
                    return Err(ParseError::with_msg(
                        "could not match str as there are none left in the source",
                    ));
                }
                Ok(Some((i, c))) => {
                    if c != next_char {
                        self.reset_pointer_loc();
                        return Ok(false);
                    }
                    match match_iter.next() {
                        Some(c) => next_char = c,
                        None => {
                            self.consume(i)?;
                            return Ok(true);
                        }
                    }
                }
            }
        }
    }

    fn match_num<N: Display + PartialEq + PartialOrd + Add<N> + Sub<N> + Mul<N> + Rem<N>>(
        &mut self,
        val: N,
    ) -> ParseResult<bool> {
        self.match_str(format!("{}", val).as_str())
    }

    fn consume_whitespace(&mut self) -> ParseResult<()> {
        if self.get_pointer_loc() != 0 {
            return Err(ParseError::with_msg(format!("Parser has already been used, and has left a pointer at position {} (which should be 0).", self.get_pointer_loc())));
        }
        loop {
            match self.peek() {
                Err(e) => {
                    return Err(e
                        .make_inner()
                        .msg("could not consume whitespace")
                        .context(self.get_context()));
                }
                Ok(None) => {
                    return Ok(());
                }
                Ok(Some((_, c))) => {
                    if c.is_whitespace() {
                        self.next().map_err(|e| {
                            e.make_inner()
                                .msg("could not consume whitespace")
                                .context(self.get_context())
                        })?;
                    } else {
                        self.consume(self.get_pointer_loc()).map_err(|e| {
                            e.make_inner()
                                .msg("could not consume whitespace")
                                .context(self.get_context())
                        })?;
                        return Ok(());
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::StrSource;
    #[test]
    fn parser_func_tests() {
        let mut source = StrSource::new(
            "This is a word. And some \"Strings, amazing!\" 1 -2 12.3 (Or something like that) 2!",
        );
        assert_eq!(source.parse_word().unwrap(), "This".to_owned());
        assert_eq!(source.parse_word().unwrap(), "is".to_owned());
        assert_eq!(source.parse_word().unwrap(), "a".to_owned());
        assert_eq!(source.parse_word().unwrap(), "word".to_owned());
        assert_eq!(source.parse_symbol().unwrap(), '.');
        assert_eq!(source.parse_word().unwrap(), "And".to_owned());
        assert_eq!(source.parse_word().unwrap(), "Some".to_owned());
        assert_eq!(
            source.parse_string().unwrap(),
            "Strings, amazing!".to_owned()
        );
        assert_eq!(source.parse_num::<u32>().unwrap(), 1);
        assert_eq!(source.parse_num::<i32>().unwrap(), -2);
        assert_eq!(source.parse_num::<f32>().unwrap(), 12.3);
        assert_eq!(
            source.parse_brackets().unwrap(),
            "Or something like that".to_owned()
        );
        assert_eq!(source.parse_num::<i64>().unwrap(), 2);
        assert_eq!(source.parse_symbol().unwrap(), '!');
    }
}
