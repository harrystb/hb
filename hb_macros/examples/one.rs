use hb_macros::{context};
use hb_parse::error::{ParseResult, ParseError};
struct Example;

impl Example {
    #[context("Basic Parse Error")]
    pub fn basic_parseerror() -> ParseResult<()> {
        Example::ParseError()
    }
    
    //#[context("Returned Parse Error")]
    //pub fn returned_parseerror() -> ParseResult<()> {
        //return Example::ParseError()
    //}

    #[context("Question Mark Parse Error")]
    pub fn question_parseerror() -> ParseResult<()> {
        Example::ParseError()?;
        Ok(())
    }

    #[context("Basic IO Error")]
    pub fn basic_ioerror() -> ParseResult<()> {
        Example::IOError()
    }
    
    //#[context("Returned IO Error")]
    //pub fn returned_ioerror() -> ParseResult<()> {
        //return Example::IOError();
    //}

    #[context("Question Mark IO Error")]
    pub fn question_ioerror() -> ParseResult<()> {
        Example::IOError()?;
        Ok(())
    }

    pub fn IOError() -> std::io::Result<()> {
        Err(std::io::Error::new(std::io::ErrorKind::AlreadyExists, "Example IO Error."))
    }

    pub fn ParseError() -> ParseResult<()> {
        Err(ParseError::with_msg("Example ParseError."))
    }
}

fn main() {
    println!("{:?}", Example::basic_parseerror());
    //println!("{}", Example::returned_parseerror());
    println!("{:?}", Example::question_parseerror());
    println!("{:?}", Example::basic_ioerror());
    //println!("{}", Example::returned_ioerror());
    println!("{:?}", Example::question_ioerror());
}