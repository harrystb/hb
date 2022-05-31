#[derive(PartialEq)]
pub struct ParseError {
    msg: String,
}
impl ParseError {
    pub fn new(msg: String) -> ParseError {
        ParseError { msg: msg }
    }

    pub fn with_msg<S: Into<String>>(msg: S) -> ParseError {
        return ParseError::new(msg.into());
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

pub type SourceResult<T> = Result<T, SourceError>;

pub enum SourceErrorType {
    IOError(std::io::Error),
    Other,
}
impl std::fmt::Display for SourceErrorType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        match self {
            SourceErrorType::IOError(e) => write!(f, "inner error:\n {}", e)?,
            SourceErrorType::Other => (),
        };
        Ok(())
    }
}

impl std::fmt::Debug for SourceErrorType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        match self {
            SourceErrorType::IOError(e) => write!(f, "inner error:\n {:?}", e)?,
            SourceErrorType::Other => (),
        };
        Ok(())
    }
}

pub struct SourceError {
    msg: String,
    inner_error: SourceErrorType,
}

impl SourceError {
    pub fn new(msg: String) -> SourceError {
        SourceError {
            msg: msg,
            inner_error: SourceErrorType::Other,
        }
    }

    pub fn from_io_error(msg: String, e: std::io::Error) -> SourceError {
        SourceError {
            msg: msg,
            inner_error: SourceErrorType::IOError(e),
        }
    }

    pub fn with_msg<S: Into<String>>(msg: S) -> SourceError {
        return SourceError::new(msg.into());
    }
}
impl std::fmt::Display for SourceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        write!(f, "Source Error: '{}' {}", self.msg, self.inner_error)?;
        Ok(())
    }
}
impl std::fmt::Debug for SourceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        write!(f, "Source Error: '{}' {}", self.msg, self.inner_error)?;
        Ok(())
    }
}
