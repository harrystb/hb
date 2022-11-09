use crate::error::{ParseError, ParseResult, UnexpectedChar};
use crate::source::Source;
use crate::SourceEmpty;
use hb_error::{context, ErrorContext};
use std::any::TypeId;
use std::fmt::Display;
use std::ops::{Add, Mul, Rem, Sub};
use std::str::FromStr;

// Trait to mark number types that can be parsed
pub trait ParsableInts {}
macro_rules! trait_parse_int {
    ($($t:ty), *) => {$(
        impl ParsableInts for $t {}
    )*}
}
trait_parse_int!(i8, i16, i32, i64, i128, u8, u16, u32, u64, u128, usize, isize);
// Trait to mark number types that can be parsed
pub trait ParsableFloats {}
macro_rules! trait_parse_float {
    ($($t:ty), *) => {$(
        impl ParsableFloats for $t {}
    )*}
}
trait_parse_float!(f32, f64);
// Trait to mark number types that can be parsed
pub trait ParsableNums {}
macro_rules! trait_parse_num {
    ($($t:ty), *) => {$(
        impl ParsableNums for $t {}
    )*}
}
trait_parse_num!(i8, i16, i32, i64, i128, u8, u16, u32, u64, u128, f32, f64, usize, isize);

pub trait CommonParserFunctions {
    // Check functions are the basic level which checks the upcoming chars and moves the pointer if
    // it is there. They are used by the other functions.
    /// Checks the upcoming chars for a word and moves the cursor if found.
    /// A word is a all alphanumeric characters leading up to a non-alphanumeric character.
    fn check_word(&mut self) -> ParseResult<bool>;
    /// Checks the upcoming chars for a string and moves the cursor if found.
    /// A string is either a word, or a set of chars enclosed by " or '.
    fn check_string(&mut self) -> ParseResult<bool>;
    /// Checks the upcoming chars for opening bracket, content and closing bracket and moves the cursor if found.
    fn check_bracket_contents(&mut self) -> ParseResult<bool>;
    /// Checks the upcoming chars for a float and moves the cursor if found.
    fn check_float(&mut self) -> ParseResult<bool>;
    /// Checks the upcoming chars for a integer number and moves the cursor if found.
    fn check_num(&mut self) -> ParseResult<bool>;
    /// Checks the upcoming chars for a symbol and moves the cursor if found.
    /// A symbol is defined as non-alphanumeric and non-whitespace.
    fn check_symbol(&mut self) -> ParseResult<bool>;

    // Parse functions build on top of the check function but also return the item found.
    // The cursor is moved and the internal buffer is not shifted.
    /// Parses a word from the upcoming chars.
    /// A word is a all alphanumeric characters leading up to a non-alphanumeric character.
    fn parse_word(&mut self) -> ParseResult<String>;
    /// Parses a string from the upcoming chars..
    /// A string is either a word, or a set of chars enclosed by " or '.
    fn parse_string(&mut self) -> ParseResult<String>;
    /// Parses the contents of some brackets from the upcoming chars.
    fn parse_brackets(&mut self) -> ParseResult<String>;
    /// Parses a float from the upcoming chars.
    fn parse_float<N: ParsableNums + ParsableFloats + std::str::FromStr>(
        &mut self,
    ) -> ParseResult<N>;
    /// Parses a integer from the upcoming chars.
    fn parse_num<N: ParsableNums + ParsableInts + std::str::FromStr>(&mut self) -> ParseResult<N>;
    /// Parses a symbol from the upcoming chars.
    /// A symbol is defined as non-alphanumeric and non-whitespace.
    fn parse_symbol(&mut self) -> ParseResult<char>;

    // Read functions build on the parse functions but also shift the internal buffer.
    /// Reads a word from the upcoming chars.
    /// A word is a all alphanumeric characters leading up to a non-alphanumeric character.
    fn read_word(&mut self) -> ParseResult<String>;
    /// Reads a string from the upcoming chars.
    /// A string is either a word, or a set of chars enclosed by " or '.
    fn read_string(&mut self) -> ParseResult<String>;
    /// Reads a the contents of some brackets from the upcoming chars.
    fn read_bracket_contents(&mut self) -> ParseResult<String>;
    /// Reads a float from the upcoming chars.
    fn read_float<N: ParsableNums + ParsableFloats + std::str::FromStr>(
        &mut self,
    ) -> ParseResult<N>;
    /// Reads a integer (of the type provided) from the upcoming chars.
    fn read_num<N: ParsableNums + ParsableInts + std::str::FromStr>(&mut self) -> ParseResult<N>;
    /// Reads a symbol from the upcoming chars.
    /// A symbol is defined as non-alphanumeric and non-whitespace.
    fn read_symbol(&mut self) -> ParseResult<char>;

    // Match functions use the parse function to extract some data and then compares it to the
    // value provided.
    /// Matches the upcoming chars against the provided char.
    fn match_char(&mut self, val: char) -> ParseResult<bool>;
    /// Checks the upcoming chars if they match the str value provided.
    fn match_str(&mut self, val: &str) -> ParseResult<bool>;
    /// Matches the upcoming chars for bracketed contents and if that matches the str provided.
    fn match_bracket_contents(&mut self, val: &str) -> ParseResult<bool>;
    /// Matches the upcoming chars against the provided number.
    fn match_num<N: ParsableInts + Display + std::str::FromStr>(
        &mut self,
        val: N,
    ) -> ParseResult<bool>;
    /// Matches the upcoming chars against the provided float.
    fn match_float<N: ParsableFloats + Display + std::str::FromStr>(
        &mut self,
        val: N,
    ) -> ParseResult<bool>;
    /// Matches the upcoming chars against the provided symbol character.
    fn match_symbol(&mut self, val: char) -> ParseResult<bool>;

    // Utility functions
    fn consume_whitespace(&mut self) -> ParseResult<()>;
    fn skip_whitespace(&mut self) -> ParseResult<()>;
}

impl<T: Source> CommonParserFunctions for T {
    #[context("could not read word")]
    fn read_word(&mut self) -> ParseResult<String> {
        self.skip_whitespace()?;
        let start_i = self.get_pointer_loc();
        loop {
            match self.peek() {
                Err(e) => {
                    self.reset_pointer_loc();
                    return Err(e);
                }
                Ok(None) => {
                    if self.get_pointer_loc() == start_i {
                        self.reset_pointer_loc();
                        return Err(SourceEmpty::new());
                    }
                    let r = self.read_substr(start_i, self.get_pointer_loc() - start_i)?;
                    return Ok(r);
                }
                Ok(Some((i, c))) => {
                    if !c.is_alphanumeric() {
                        return self.read_substr(start_i, i - start_i);
                    } else {
                        self.next()?;
                    }
                }
            }
        }
    }

    #[context("could not parse word")]
    fn parse_word(&mut self) -> ParseResult<String> {
        if self.get_pointer_loc() != 0 {
            return Err(ParseError::new().msg(format!("Parser has already been used, and has left a pointer at position {} (which should be 0).", self.get_pointer_loc())));
        }
        let word = self.read_word()?;
        self.consume(self.get_pointer_loc())?;
        Ok(word)
    }

    #[context("could not parse string")]
    fn parse_string(&mut self) -> ParseResult<String> {
        if self.get_pointer_loc() != 0 {
            return Err(ParseError::new().msg(format!("Parser has already been used, and has left a pointer at position {} (which should be 0).", self.get_pointer_loc())));
        }
        self.skip_whitespace()?;
        let start_i = self.get_pointer_loc();
        let expected_ending;
        match self.next() {
            Err(e) => {
                self.reset_pointer_loc();
                return Err(e);
            }
            Ok(None) => {
                self.reset_pointer_loc();
                return Err(SourceEmpty::new());
            }
            Ok(Some((_, c))) => {
                if c == '\'' {
                    expected_ending = '\'';
                } else if c == '"' {
                    expected_ending = '"';
                } else {
                    return self.parse_word();
                }
            }
        }
        loop {
            match self.next() {
                Err(e) => {
                    self.reset_pointer_loc();
                    return Err(e);
                }
                Ok(None) => {
                    self.reset_pointer_loc();
                    return Err(SourceEmpty::new());
                }
                Ok(Some((i, c))) => {
                    if c == expected_ending {
                        if i == 1 {
                            return Ok(String::new());
                        }
                        let ret = Ok(self.read_substr(start_i + 1, i - start_i - 1)?);
                        self.consume(i + 1)?;
                        return ret;
                    }
                }
            }
        }
    }

    #[context("could not parse brackets")]
    fn parse_brackets(&mut self) -> ParseResult<String> {
        if self.get_pointer_loc() != 0 {
            return Err(ParseError::new().msg(format!("Parser has already been used, and has left a pointer at position {} (which should be 0).", self.get_pointer_loc())));
        }
        self.skip_whitespace()?;
        let start_i = self.get_pointer_loc();
        let expected_ending;
        let mut level = 1;
        match self.next() {
            Err(e) => {
                self.reset_pointer_loc();
                return Err(e);
            }
            Ok(None) => {
                self.reset_pointer_loc();
                return Err(SourceEmpty::new());
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
                    return Err(UnexpectedChar::new().msg(format!(
                        "'{}' was found instead of a bracket (either (, [, < or {{)",
                        c
                    )));
                }
            }
        }
        loop {
            match self.next() {
                Err(e) => {
                    self.reset_pointer_loc();
                    return Err(e);
                }
                Ok(None) => {
                    self.reset_pointer_loc();
                    return Err(SourceEmpty::new());
                }
                Ok(Some((i, c))) => {
                    if c == expected_ending {
                        level -= 1;
                        if level == 0 {
                            let ret = Ok(self.read_substr(start_i + 1, i - start_i - 1)?);
                            self.consume(i + 1)?;
                            return ret;
                        }
                    }
                }
            }
        }
    }

    #[context("could not parse num")]
    fn parse_num<N: ParsableNums + ParsableInts + std::str::FromStr>(&mut self) -> ParseResult<N> {
        if self.get_pointer_loc() != 0 {
            return Err(ParseError::new().msg(format!("Parser has already been used, and has left a pointer at position {} (which should be 0).", self.get_pointer_loc())));
        }
        self.skip_whitespace()?;
        let start_i = self.get_pointer_loc();
        // skip a +/-
        match self.peek()? {
            None => {
                return Err(SourceEmpty::new());
            }
            Some((_, c)) => {
                if c == '-' || c == '+' {
                    self.next()?;
                }
            }
        }
        while let Some((_, c)) = self.peek()? {
            if !c.is_ascii_digit() {
                break;
            } else {
                self.next()?;
            }
        }
        let substr = self
            .read_substr(start_i, self.get_pointer_loc() - start_i)
            .unwrap();
        match substr.parse::<N>() {
            Err(_) => {
                self.reset_pointer_loc();
                return Err(ParseError::new().msg(format!("'{}' is not a valid number", substr)));
            }
            Ok(n) => {
                self.consume(self.get_pointer_loc())?;
                Ok(n)
            }
        }
    }
    #[context("could not parse num")]
    fn parse_float<N: ParsableNums + ParsableFloats + std::str::FromStr>(
        &mut self,
    ) -> ParseResult<N> {
        if self.get_pointer_loc() != 0 {
            return Err(ParseError::new().msg(format!("Parser has already been used, and has left a pointer at position {} (which should be 0).", self.get_pointer_loc())));
        }
        self.skip_whitespace()?;
        // Need to allow any of the below for float type numbers.
        //Float  ::= Sign? ( 'inf' | 'infinity' | 'nan' | Number )
        //Number ::= ( Digit+ |
        //'.' Digit* |
        //Digit+ '.' Digit* |
        //Digit* '.' Digit+ ) Exp?
        //Exp    ::= 'e' Sign? Digit+
        //Sign   ::= [+-]
        //Digit  ::= [0-9]
        let start_i = self.get_pointer_loc();
        // skip a +/-
        match self.peek()? {
            None => {
                return Err(SourceEmpty::new());
            }
            Some((_, c)) => {
                if c == '-' || c == '+' {
                    self.next()?;
                }
            }
        }
        // shortcut inf infinity or nan
        let mut is_shortcut = false;
        match self.peek()? {
            None => {
                return Err(SourceEmpty::new());
            }
            Some((_, c)) => {
                if c == 'i' || c == 'I' || c == 'n' || c == 'N' {
                    let mut word = self.read_word().map_err(|e| {
                        e.make_inner().msg(format!(
                            "could not finish the infinity or not a number word after {}",
                            c
                        ))
                    })?;
                    word.make_ascii_uppercase();
                    if word == "INF" || word == "INFINITY" || word == "NAN" {
                        is_shortcut = true;
                    }
                }
            }
        }
        // process inf infinity and nan immediately
        if is_shortcut {
            let substr = self
                .read_substr(start_i, self.get_pointer_loc() - start_i)
                .unwrap();
            match substr.parse::<N>() {
                Err(_) => {
                    self.reset_pointer_loc();
                    return Err(ParseError::new().msg(format!("'{}' is not a valid float", substr)));
                }
                Ok(n) => {
                    self.consume(self.get_pointer_loc())?;
                    return Ok(n);
                }
            }
        }
        // skip digits,
        while let Some((_, c)) = self.peek()? {
            //already handled any non-digit cases above so can
            if !c.is_ascii_digit() {
                break;
            } else {
                self.next()?;
            }
        }
        // if '.' then skip more digits
        if let Some((_, c)) = self.peek()? {
            //already handled any non-digit cases above so can
            if c == '.' {
                self.next()?;
            }
        }
        while let Some((_, c)) = self.peek()? {
            //already handled any non-digit cases above so can
            if !c.is_ascii_digit() {
                break;
            } else {
                self.next()?;
            }
        }
        // if skip 'e' and skip sign more digits.
        let mut has_exp = false;
        if let Some((_, c)) = self.peek()? {
            //already handled any non-digit cases above so can
            if c == 'e' || c == 'E' {
                has_exp = true;
                self.next()?;
            }
        }
        if has_exp {
            if let Some((_, c)) = self.peek()? {
                //already handled any non-digit cases above so can
                if c == '+' || c == '-' {
                    self.next()?;
                }
            }
            while let Some((_, c)) = self.peek()? {
                //already handled any non-digit cases above so can
                if !c.is_ascii_digit() {
                    break;
                } else {
                    self.next()?;
                }
            }
        }

        let substr = self
            .read_substr(start_i, self.get_pointer_loc() - start_i)
            .unwrap();
        match substr.parse::<N>() {
            Err(_) => {
                self.reset_pointer_loc();
                return Err(ParseError::new().msg(format!("'{}' is not a valid number", substr)));
            }
            Ok(n) => {
                self.consume(self.get_pointer_loc())?;
                Ok(n)
            }
        }
    }

    #[context("could not parse symbol")]
    fn parse_symbol(&mut self) -> ParseResult<char> {
        if self.get_pointer_loc() != 0 {
            return Err(ParseError::new().msg(format!("Parser has already been used, and has left a pointer at position {} (which should be 0).", self.get_pointer_loc())));
        }
        self.read_symbol()
    }

    #[context("could not read symbol")]
    fn read_symbol(&mut self) -> ParseResult<char> {
        self.skip_whitespace()?;
        match self.peek()? {
            None => Err(SourceEmpty::new().into()),
            Some((_, c)) => {
                if !c.is_whitespace() && !c.is_ascii_alphanumeric() {
                    // remove the char from the source
                    self.next()?;
                    Ok(c)
                } else {
                    return Err(
                        ParseError::new().msg(format!("'{}' is not classified as a symbol", c))
                    );
                }
            }
        }
    }

    #[context("could not match char {val}")]
    fn match_char(&mut self, val: char) -> ParseResult<bool> {
        if self.get_pointer_loc() != 0 {
            return Err(ParseError::new().msg(format!("Parser has already been used, and has left a pointer at position {} (which should be 0).", self.get_pointer_loc())));
        }
        self.skip_whitespace()?;
        match self.peek()? {
            None => Err(SourceEmpty::new().into()),
            Some((i, c)) => {
                if c == val {
                    // remove the char from the source
                    self.consume(i + 1)?;
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
        }
    }

    #[context("could not match num {val}")]
    fn match_num<N: ParsableNums + Display + std::str::FromStr>(
        &mut self,
        val: N,
    ) -> ParseResult<bool> {
        self.match_str(format!("{}", val).as_str())
    }

    #[context("could not match str {val}")]
    fn match_str(&mut self, val: &str) -> ParseResult<bool> {
        if self.get_pointer_loc() != 0 {
            return Err(ParseError::new().msg(format!("Parser has already been used, and has left a pointer at position {} (which should be 0).", self.get_pointer_loc())));
        }
        self.skip_whitespace()?;
        let mut match_iter = val.chars();
        let mut next_char = match match_iter.next() {
            Some(c) => c,
            None => {
                return Err(SourceEmpty::new());
            }
        };
        loop {
            match self.next()? {
                None => {
                    self.reset_pointer_loc();
                    return Err(SourceEmpty::new());
                }
                Some((i, c)) => {
                    if c != next_char {
                        self.reset_pointer_loc();
                        return Ok(false);
                    }
                    match match_iter.next() {
                        Some(c) => next_char = c,
                        None => {
                            self.consume(i + 1)?;
                            return Ok(true);
                        }
                    }
                }
            }
        }
    }

    #[context("could not consume whitespace")]
    fn consume_whitespace(&mut self) -> ParseResult<()> {
        if self.get_pointer_loc() != 0 {
            return Err(ParseError::new().msg(format!("Parser has already been used, and has left a pointer at position {} (which should be 0).", self.get_pointer_loc())));
        }
        loop {
            match self.peek()? {
                None => {
                    return Ok(());
                }
                Some((_, c)) => {
                    if c.is_whitespace() {
                        self.next()?;
                    } else {
                        self.consume(self.get_pointer_loc())?;
                        return Ok(());
                    }
                }
            }
        }
    }

    #[context("could not skip whitespace")]
    fn skip_whitespace(&mut self) -> ParseResult<()> {
        loop {
            match self.peek()? {
                None => {
                    return Ok(());
                }
                Some((_, c)) => {
                    if c.is_whitespace() {
                        self.next()?;
                    } else {
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
    use crate::StrParser;
    #[test]
    fn parser_func_tests() {
        let mut source = StrParser::new(
            "This is a word. And some \"Strings, amazing!\" 1 -2 12.3 +inf -infinity infinity -nan (Or something like that) 2! 1.0",
        );
        assert_eq!(source.parse_word().unwrap(), "This".to_owned());
        assert_eq!(source.parse_word().unwrap(), "is".to_owned());
        assert_eq!(source.parse_word().unwrap(), "a".to_owned());
        assert_eq!(source.parse_word().unwrap(), "word".to_owned());
        assert_eq!(source.parse_symbol().unwrap(), '.');
        source.consume_whitespace().ok();
        assert_eq!(source.parse_word().unwrap(), "And".to_owned());
        assert_eq!(source.parse_word().unwrap(), "some".to_owned());
        assert_eq!(
            source.parse_string().unwrap(),
            "Strings, amazing!".to_owned()
        );
        assert_eq!(source.parse_num::<u32>().unwrap(), 1);
        assert_eq!(source.parse_num::<i32>().unwrap(), -2);
        assert_eq!(source.parse_float::<f32>().unwrap(), 12.3);
        assert_eq!(source.parse_float::<f32>().unwrap(), f32::INFINITY);
        assert_eq!(source.parse_float::<f32>().unwrap(), f32::NEG_INFINITY);
        assert_eq!(source.parse_float::<f32>().unwrap(), f32::INFINITY);
        assert!(source.parse_float::<f32>().unwrap().is_nan());
        assert_eq!(
            source.parse_brackets().unwrap(),
            "Or something like that".to_owned()
        );
        assert_eq!(source.parse_num::<i64>().unwrap(), 2);
        assert_eq!(source.parse_symbol().unwrap(), '!');
        assert_eq!(source.parse_num::<i64>().unwrap(), 1);
        assert_eq!(source.parse_symbol().unwrap(), '.');
        assert_eq!(source.parse_num::<i64>().unwrap(), 0);
    }
}
