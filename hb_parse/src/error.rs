pub type ParseResult<T> = Result<T, ParseError>;

pub enum ParseInnerError {
    Parse(Box<ParseError>),
    IO(std::io::Error),
}
impl std::fmt::Display for ParseInnerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        match self {
            ParseInnerError::IO(e) => write!(f, "inner error:\n {}", e)?,
            ParseInnerError::Parse(e) => write!(f, "inner error:\n {}", e)?,
        };
        Ok(())
    }
}

impl std::fmt::Debug for ParseInnerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        match self {
            ParseInnerError::IO(e) => write!(f, "inner error:\n {}", e)?,
            ParseInnerError::Parse(e) => write!(f, "inner error:\n {}", e)?,
        };
        Ok(())
    }
}
pub struct ParseError {
    msg: String,
    inner_error: Option<ParseInnerError>,
}

impl ParseError {
    pub fn new(msg: String) -> ParseError {
        ParseError {
            msg: msg,
            inner_error: None,
        }
    }

    pub fn with_msg<S: Into<String>>(msg: S) -> ParseError {
        ParseError::new(msg.into())
    }

    pub fn with_context<S: Into<String>>(e: ParseInnerError, msg: S) -> ParseError {
        ParseError {
            msg: msg.into(),
            inner_error: Some(e),
        }
    }
}
impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        write!(f, "Parseing Error: '{}'", self.msg)?;
        Ok(())
    }
}
impl std::fmt::Debug for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        write!(f, "Parseing Error: '{}'", self.msg)?;
        Ok(())
    }
}
