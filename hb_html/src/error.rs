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
