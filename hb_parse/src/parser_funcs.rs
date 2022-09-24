use crate::error::{ParseError, ParseResult};
use crate::source::Source;
use crate::SourceEmpty;
use hb_error::context;
use std::any::TypeId;
use std::fmt::Display;
use std::ops::{Add, Mul, Rem, Sub};
use std::str::FromStr;

// TODO: Make trait for num functions and use macro to impl for it
macro_rules! trait_fns_parse_num {
    ($($t:ty), *) => {$(
        #[doc(parses a number of type $t)]
        fn parse_num<$t>(&mut self) -> ParseResult<$t>;
    )*}
}
macro_rules! trait_fns_match_num {
    ($($t:ty), *) => {$(
        #[doc(Checks upcoming chars if they match the value provided of type $t)]
        fn match_num<$t>(&mut self) -> ParseResult<$t>;
    )*}
}

pub trait CommonParserFunctions {
    /// Reads a word from the source from the current cursor.
    /// A word is a all alphanumberic characters leading up to a non-aplphanumeric character.
    fn read_word(&mut self) -> ParseResult<String>;
    /// Parses a word from the source.
    /// A word is a all alphanumberic characters leading up to a non-aplphanumeric character.
    fn parse_word(&mut self) -> ParseResult<String>;
    /// Parses a string from the source.
    /// A string is either a word, or a set of chars enclosed by " or '.
    fn parse_string(&mut self) -> ParseResult<String>;
    /// Parses the string until the other end of the brackets is found.
    fn parse_brackets(&mut self) -> ParseResult<String>;
    // Parses a number (eg i32, i64, u32, u64, f32, f64)
    trait_fns_parse_num!(i8, i16, i32, i64, i128, u8, u16, u32, u64, u128, f32, f64, usize, isize);
    /// Parses a symbol which is defined as non-alphanumeric and non-whitespace.
    fn parse_symbol(&mut self) -> ParseResult<char>;
    fn read_symbol(&mut self) -> ParseResult<char>;
    /// Checks if the next char matches the provided value.
    /// If it matches then the parser is moved forward.
    fn match_char(&mut self, val: char) -> ParseResult<bool>;
    /// Checks the upcoming chars if they match the int value provided.
    /// If it matches then the parser is moved forward.
    trait_fns_match_num!(i8, i16, i32, i64, i128, u8, u16, u32, u64, u128, f32, f64, usize, isize);
    /// Checks the upcoming chars if they match the str value provided.
    /// If it matches then the parser is moved forward.
    fn match_str(&mut self, val: &str) -> ParseResult<bool>;
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
                        return Err(ParseError::new()
                            .msg("could not parse word as there are none left in the source")
                            .err_type_source_empty());
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
                    return Ok(self.parse_word())?;
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
    fn parse_num<
        N: 'static + FromStr + PartialEq + PartialOrd + Add<N> + Sub<N> + Mul<N> + Rem<N>,
    >(
        &mut self,
    ) -> ParseResult<N> {
        if self.get_pointer_loc() != 0 {
            return Err(ParseError::new().msg(format!("Parser has already been used, and has left a pointer at position {} (which should be 0).", self.get_pointer_loc())));
        }
        self.skip_whitespace().map_err(|e| {
            e.make_inner()
                .msg("could not parse num")
                .context(self.get_context())
        })?;
        let is_float =
            TypeId::of::<N>() == TypeId::of::<f64>() || TypeId::of::<N>() == TypeId::of::<f32>();

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
        match self.peek().map_err(|e| {
            e.make_inner()
                .msg("could not parse num")
                .context(self.get_context())
        })? {
            None => {
                return Err(ParseError::new()
                    .msg("could not parse num as there are none left in the source"));
            }
            Some((_, c)) => {
                if c == '-' || c == '+' {
                    self.next()?;
                }
            }
        }
        // shortcut inf infinity or nan
        let mut is_shortcut = false;
        match self.peek().map_err(|e| {
            e.make_inner()
                .msg("could not parse num")
                .context(self.get_context())
        })? {
            None => {
                return Err(ParseError::new()
                    .msg("could not parse num as there are none left in the source"));
            }
            Some((_, c)) => {
                if is_float && (c == 'i' || c == 'I' || c == 'n' || c == 'N') {
                    let mut word = self.read_word().map_err(|e| {
                        e.make_inner()
                            .msg(format!("could not parse float after {}", c))
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
                    return Err(
                        ParseError::new().msg(format!("could not parse num from str '{}'", substr))
                    );
                }
                Ok(n) => {
                    self.consume(self.get_pointer_loc())?;
                    return Ok(n);
                }
            }
        }
        // skip digits,
        while let Some((_, c)) = self.peek().map_err(|e| {
            e.make_inner()
                .msg("could not parse num")
                .context(self.get_context())
        })? {
            //already handled any non-digit cases above so can
            if !c.is_ascii_digit() {
                break;
            } else {
                self.next()?;
            }
        }
        if is_float {
            // if '.' then skip more digits
            if let Some((_, c)) = self.peek().map_err(|e| {
                e.make_inner()
                    .msg("could not parse num")
                    .context(self.get_context())
            })? {
                //already handled any non-digit cases above so can
                if c == '.' {
                    self.next()?;
                }
            }
            while let Some((_, c)) = self.peek().map_err(|e| {
                e.make_inner()
                    .msg("could not parse num")
                    .context(self.get_context())
            })? {
                //already handled any non-digit cases above so can
                if !c.is_ascii_digit() {
                    break;
                } else {
                    self.next()?;
                }
            }
            // if skip 'e' and skip sign more digits.
            let mut has_exp = false;
            if let Some((_, c)) = self.peek().map_err(|e| {
                e.make_inner()
                    .msg("could not parse num")
                    .context(self.get_context())
            })? {
                //already handled any non-digit cases above so can
                if c == 'e' || c == 'E' {
                    has_exp = true;
                    self.next()?;
                }
            }
            if has_exp {
                if let Some((_, c)) = self.peek().map_err(|e| {
                    e.make_inner()
                        .msg("could not parse num")
                        .context(self.get_context())
                })? {
                    //already handled any non-digit cases above so can
                    if c == '+' || c == '-' {
                        self.next()?;
                    }
                }
                while let Some((_, c)) = self.peek().map_err(|e| {
                    e.make_inner()
                        .msg("could not parse num")
                        .context(self.get_context())
                })? {
                    //already handled any non-digit cases above so can
                    if !c.is_ascii_digit() {
                        break;
                    } else {
                        self.next()?;
                    }
                }
            }
        }

        let substr = self
            .read_substr(start_i, self.get_pointer_loc() - start_i)
            .unwrap();
        match substr.parse::<N>() {
            Err(_) => {
                self.reset_pointer_loc();
                return Err(
                    ParseError::new().msg(format!("could not parse num from str '{}'", substr))
                );
            }
            Ok(n) => {
                self.consume(self.get_pointer_loc())?;
                Ok(n)
            }
        }
    }

    fn read_symbol(&mut self) -> ParseResult<char> {
        self.skip_whitespace().map_err(|e| {
            e.make_inner()
                .msg("could not parse symbol")
                .context(self.get_context())
        })?;
        match self.peek() {
            Err(e) => Err(ParseError::with_context(
                ParseInnerError::Parse(Box::new(e)),
                "could not parse symbol",
            )),
            Ok(None) => Err(ParseError::new()
                .msg("could not parse symbol as there are none left in the source")
                .err_type_source_empty()),
            Ok(Some((_, c))) => {
                if !c.is_whitespace() && !c.is_ascii_alphanumeric() {
                    // remove the char from the source
                    self.next()?;
                    Ok(c)
                } else {
                    return Err(ParseError::new().msg(format!(
                        "cound not parse symbol because '{}' is not classified as a symbol",
                        c
                    )));
                }
            }
        }
    }

    fn parse_symbol(&mut self) -> ParseResult<char> {
        if self.get_pointer_loc() != 0 {
            return Err(ParseError::new().msg(format!("Parser has already been used, and has left a pointer at position {} (which should be 0).", self.get_pointer_loc())));
        }
        self.read_symbol()
    }

    fn match_char(&mut self, val: char) -> ParseResult<bool> {
        if self.get_pointer_loc() != 0 {
            return Err(ParseError::new().msg(format!("Parser has already been used, and has left a pointer at position {} (which should be 0).", self.get_pointer_loc())));
        }
        self.skip_whitespace().map_err(|e| {
            e.make_inner()
                .msg(format!("could not match char {}", val))
                .context(self.get_context())
        })?;
        match self.peek() {
            Err(e) => Err(ParseError::with_context(
                ParseInnerError::Parse(Box::new(e)),
                "could not match char",
            )),
            Ok(None) => Err(ParseError::new()
                .msg("could not match char as there are none left in the source")
                .err_type_source_empty()),
            Ok(Some((i, c))) => {
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

    fn match_str(&mut self, val: &str) -> ParseResult<bool> {
        if self.get_pointer_loc() != 0 {
            return Err(ParseError::new().msg(format!("Parser has already been used, and has left a pointer at position {} (which should be 0).", self.get_pointer_loc())));
        }
        self.skip_whitespace().map_err(|e| {
            e.make_inner()
                .msg(format!("could not match str {}", val))
                .context(self.get_context())
        })?;
        let mut match_iter = val.chars();
        let mut next_char = match match_iter.next() {
            Some(c) => c,
            None => {
                return Err(ParseError::new().msg("cannot match str as the str provided is empty"))
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
                    return Err(ParseError::new()
                        .msg("could not match str as there are none left in the source")
                        .err_type_source_empty());
                }
                Ok(Some((i, c))) => {
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

    fn match_num<N: Display + PartialEq + PartialOrd + Add<N> + Sub<N> + Mul<N> + Rem<N>>(
        &mut self,
        val: N,
    ) -> ParseResult<bool> {
        self.match_str(format!("{}", val).as_str())
    }

    fn consume_whitespace(&mut self) -> ParseResult<()> {
        if self.get_pointer_loc() != 0 {
            return Err(ParseError::new().msg(format!("Parser has already been used, and has left a pointer at position {} (which should be 0).", self.get_pointer_loc())));
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

    fn skip_whitespace(&mut self) -> ParseResult<()> {
        loop {
            match self.peek() {
                Err(e) => {
                    return Err(e
                        .make_inner()
                        .msg("could not skip whitespace")
                        .context(self.get_context()));
                }
                Ok(None) => {
                    return Ok(());
                }
                Ok(Some((_, c))) => {
                    if c.is_whitespace() {
                        self.next().map_err(|e| {
                            e.make_inner()
                                .msg("could not skip whitespace")
                                .context(self.get_context())
                        })?;
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
        assert_eq!(source.parse_num::<f32>().unwrap(), 12.3);
        assert_eq!(source.parse_num::<f32>().unwrap(), f32::INFINITY);
        assert_eq!(source.parse_num::<f32>().unwrap(), f32::NEG_INFINITY);
        assert_eq!(source.parse_num::<f32>().unwrap(), f32::INFINITY);
        assert!(source.parse_num::<f32>().unwrap().is_nan());
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
