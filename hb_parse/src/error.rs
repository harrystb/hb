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
