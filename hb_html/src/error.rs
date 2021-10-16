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

pub struct ParseHtmlError {
    msg: String,
}

impl ParseHtmlError {
    pub fn new(msg: String) -> ParseHtmlError {
        ParseHtmlError { msg: msg }
    }

    pub fn with_msg<S: Into<String>>(msg: S) -> ParseHtmlError {
        return ParseHtmlError::new(msg.into());
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
