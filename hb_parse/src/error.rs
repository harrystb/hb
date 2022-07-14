pub type ParseResult<T> = Result<T, ParseError>;

pub enum ParseInnerError {
    Parse(Box<ParseError>),
    IO(std::io::Error),
}

#[derive(PartialEq, Debug)]
pub enum ParseErrorType {
    SourceEmpty,
    Unspecified,
}

impl std::fmt::Display for ParseInnerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        match self {
            ParseInnerError::IO(e) => write!(f, "\nBECAUSE\n {}", e)?,
            ParseInnerError::Parse(e) => write!(f, "\nBECAUSE\n {}", e)?,
        };
        Ok(())
    }
}

impl std::fmt::Debug for ParseInnerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        match self {
            ParseInnerError::IO(e) => write!(f, "\nBECAUSE\n {}", e)?,
            ParseInnerError::Parse(e) => write!(f, "\nBECAUSE\n {}", e)?,
        };
        Ok(())
    }
}

pub struct ParseError {
    pub msg: String,
    pub context: String,
    pub inner_error: Option<ParseInnerError>,
    pub err_type: ParseErrorType,
}

impl ParseError {
    /// Creates a new empty error
    pub fn new() -> ParseError {
        ParseError {
            msg: String::new(),
            context: String::new(),
            inner_error: None,
            err_type: ParseErrorType::Unspecified,
        }
    }

    /// Creates a new error with the given message
    pub fn with_msg<S: Into<String>>(msg: S) -> ParseError {
        ParseError {
            msg: msg.into(),
            context: String::new(),
            inner_error: None,
            err_type: ParseErrorType::Unspecified,
        }
    }

    /// Creates a new error with the given message and inner error
    pub fn with_context<S: Into<String>>(e: ParseInnerError, msg: S) -> ParseError {
        ParseError {
            msg: msg.into(),
            context: String::new(),
            inner_error: Some(e),
            err_type: ParseErrorType::Unspecified,
        }
    }

    /// Used for taking a ParseError making it the inner error of a new ParseError.
    /// (eg error.make_inner().msg("could not do something"))
    pub fn make_inner(self) -> ParseError {
        ParseError {
            msg: String::new(),
            context: String::new(),
            inner_error: Some(ParseInnerError::Parse(Box::new(self))),
            err_type: ParseErrorType::Unspecified,
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

    pub fn err_type_source_empty(mut self) -> ParseError {
        self.err_type = ParseErrorType::SourceEmpty;
        self
    }
}

impl Default for ParseError {
    fn default() -> Self {
        Self::new()
    }
}

impl From<std::io::Error> for ParseError {
    fn from(error: std::io::Error) -> ParseError {
        ParseError {
            msg: String::new(),
            context: String::new(),
            inner_error: Some(ParseInnerError::IO(error)),
            err_type: ParseErrorType::Unspecified,
        }
    }
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        let context_str = if self.context.len() > 0{format!("\n{}", self.context)} else {String::new()};
        match &self.inner_error {
            Some(e) => write!(f, "ParseError: {}{}{}", self.msg, context_str,e),
            None => write!(f, "Parse Error: {}{}", self.msg, context_str),
        }
    }
}
impl std::fmt::Debug for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        let context_str = if self.context.len() > 0{format!("\n{}", self.context)} else {String::new()};
        match &self.inner_error {
            Some(e) => write!(f, "ParseError: {}{}{}", self.msg, context_str,e),
            None => write!(f, "Parse Error: {}{}", self.msg, context_str),
        }
    }
}