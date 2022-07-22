use hb_macros::{context, context2, context3};
use hb_parse::error::{ParseResult, ParseError};
struct eh;

impl eh {
    #[context("Something")]
    pub fn eh() -> ParseResult<()> {
        eh::b()?;
        Ok(())
    }

    #[context2("Better?")]
    pub fn hm() -> ParseResult<()> {
        Err(ParseError::with_msg("inner"))
    }

    #[context2("Better?")]
    pub fn ah() -> ParseResult<()> {
        Err(ParseError::from(std::io::Error::new(std::io::ErrorKind::AlreadyExists, "This is an io error")))
    }

    #[context3("maybe?")]
    pub fn uh() -> ParseResult<()> {
        Err(ParseError::from(std::io::Error::new(std::io::ErrorKind::AlreadyExists, "This is an io error")))
    }

    #[context3("yes?")]
    pub fn ultimate() -> ParseResult<()> {
        Err(std::io::Error::new(std::io::ErrorKind::AlreadyExists, "This is an io error"))
    }

    pub fn b() -> ParseResult<()> {
        Err(ParseError::with_msg("inner"))
    }
}

fn main() {
    println!("{}", eh::eh().err().unwrap());
    println!("{}", eh::hm().err().unwrap());
    println!("{}", eh::ah().err().unwrap());
    println!("{}", eh::uh().err().unwrap());
    println!("{}", eh::ultimate().err().unwrap());
}