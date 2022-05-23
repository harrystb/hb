use std::error::Error;

pub struct HtmlMatchError {
    msg: String,
}

impl HtmlMatchError {
    pub fn new(msg: String) -> HtmlMatchError {
        HtmlMatchError { msg: msg }
    }

    pub fn with_msg<S: Into<String>>(msg: S) -> HtmlMatchError {
        return HtmlMatchError::new(msg.into());
    }
}

impl std::fmt::Display for HtmlMatchError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        write!(f, "Html Matching Error: '{}'", self.msg)?;
        Ok(())
    }
}
impl std::fmt::Debug for HtmlMatchError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        write!(f, "Html Matching Error: '{}'", self.msg)?;
        Ok(())
    }
}

pub struct HtmlDocError {
    msg: String,
}
impl HtmlDocError {
    pub fn new(msg: String) -> HtmlDocError {
        HtmlDocError { msg: msg }
    }

    pub fn with_msg<S: Into<String>>(msg: S) -> HtmlDocError {
        return HtmlDocError::new(msg.into());
    }
}

impl std::fmt::Display for HtmlDocError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        write!(f, "Html Doc Error: '{}'", self.msg)?;
        Ok(())
    }
}
impl std::fmt::Debug for HtmlDocError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        write!(f, "Html Doc Error: '{}'", self.msg)?;
        Ok(())
    }
}

#[derive(PartialEq)]
pub struct ParseHtmlError {
    pub msg: String,
}

impl ParseHtmlError {
    pub fn new(msg: String) -> ParseHtmlError {
        ParseHtmlError { msg: msg }
    }

    pub fn with_msg<S: Into<String>>(msg: S) -> ParseHtmlError {
        return ParseHtmlError::new(msg.into());
    }

    pub fn add_context<S: Into<String>>(mut self, msg: S) -> ParseHtmlError {
        self.msg = format!("{} because {}", msg.into(), self.msg);
        self
    }
}

impl std::fmt::Display for ParseHtmlError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        write!(f, "Html Doc Error: '{}'", self.msg)?;
        Ok(())
    }
}
impl std::fmt::Debug for ParseHtmlError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        write!(f, "Html Doc Error: '{}'", self.msg)?;
        Ok(())
    }
}

impl Error for ParseHtmlError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }
}

impl Error for HtmlMatchError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }
}

impl Error for HtmlDocError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }
}
