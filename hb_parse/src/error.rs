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
    context: String,
    inner_error: Option<ParseInnerError>,
}

impl ParseError {
    /// Creates a new empty error
    pub fn new() -> ParseError {
        ParseError {
            msg: String::new(),
            context: String::new(),
            inner_error: None,
        }
    }

    /// Creates a new error with the given message
    pub fn with_msg<S: Into<String>>(msg: S) -> ParseError {
        ParseError {
            msg: msg.into(),
            context: String::new(),
            inner_error: None,
        }
    }

    /// Creates a new error with the given message and inner error
    pub fn with_context<S: Into<String>>(e: ParseInnerError, msg: S) -> ParseError {
        ParseError {
            msg: msg.into(),
            context: String::new(),
            inner_error: Some(e),
        }
    }

    /// Used for taking a ParseError making it the inner error of a new ParseError.
    /// (eg error.make_inner().msg("could not do something"))
    pub fn make_inner(self) -> ParseError {
        ParseError {
            msg: String::new(),
            context: String::new(),
            inner_error: Some(ParseInnerError::Parse(Box::new(self))),
        }
    }

    /// Used for chaining creation of a ParseError
    /// (eg ParseError::new().msg("something").context("context"))
    pub fn msg<S: Into<String>>(mut self, msg: S) -> ParseError {
        self.msg = msg.into();
        self
    }

    /// Used for adding some additional context to the error.
    /// Generally this is used for showing where a Parsing Error occurs.
    pub fn context(mut self, context: String) -> ParseError {
        self.context = context;
        self
    }
}
impl From<std::io::Error> for ParseError {
    fn from(error: std::io::Error) -> ParseError {
        ParseError {
            msg: String::new(),
            context: String::new(),
            inner_error: Some(ParseInnerError::IO(error)),
        }
    }
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        match &self.inner_error {
            Some(e) => write!(f, "Parse Error: {}\n{}\n{}", self.msg, self.context, e),
            None => write!(f, "Parse Error: {}\n{}", self.msg, self.context),
        }
    }
}
impl std::fmt::Debug for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        match &self.inner_error {
            Some(e) => write!(f, "Parse Error: {}\n{}\n{}", self.msg, self.context, e),
            None => write!(f, "Parse Error: {}\n{}", self.msg, self.context),
        }
    }
}
