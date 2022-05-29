use crate::error::ParseError;

pub trait ParseChecker {
    fn parse(&mut self, c: char) -> Result<Option<String>, ParseError>;
}

pub enum CheckerMode {
    UpTo,
    UpToAndIncluding,
    Exactly,
}

pub struct CharChecker {
    ending: char,
    buf: String,
    mode: CheckerMode,
}

impl CharChecker {
    pub fn new(ending: char, mode: CheckerMode) -> CharChecker {
        CharChecker {
            ending: ending,
            buf: String::new(),
            mode: mode,
        }
    }
}

impl<'a> ParseChecker for CharChecker {
    fn parse(&mut self, c: char) -> Result<Option<String>, ParseError> {
        match &self.mode {
            CheckerMode::UpTo => {
                if c == self.ending {
                    return Ok(Some(self.buf.clone()));
                }
                self.buf.push(c);
            }
            CheckerMode::UpToAndIncluding => {
                self.buf.push(c);
                if c == self.ending {
                    return Ok(Some(self.buf.clone()));
                }
            }
            CheckerMode::Exactly => {
                if c == self.ending {
                    return Ok(Some(self.buf.clone()));
                } else {
                    return Err(ParseError::with_msg(format!(
                        "{} does not match expected char {}",
                        c, self.ending
                    )));
                }
            }
        }
        Ok(None)
    }
}
