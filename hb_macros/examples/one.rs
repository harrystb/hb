use hb_macros::{context};
use hb_parse::error::{ParseResult, ParseError};
struct Example;

impl Example {
    #[context("Basic Parse Error")]
    pub fn basic_parseerror() -> ParseResult<()> {
        Example::ParseError()
    }
    
    #[context("Returned Parse Error")]
    pub fn returned_parseerror() -> ParseResult<()> {
        return Example::ParseError()
    }

    #[context("Question Mark Parse Error")]
    pub fn question_parseerror() -> ParseResult<()> {
        Example::ParseError()?;
        Ok(())
    }

    #[context("Basic IO Error")]
    pub fn basic_ioerror() -> ParseResult<()> {
        Ok(Example::IOError()?)
    }
    
    #[context("Returned IO Error")]
    pub fn returned_ioerror() -> ParseResult<()> {
        return Example::IOError();
    }

    #[context("Question Mark IO Error")]
    pub fn question_ioerror() -> ParseResult<()> {
        Example::IOError()?;
        Ok(())
    }

    pub fn IOError() -> std::io::Result<()> {
        Err(std::io::Error::new(std::io::ErrorKind::AlreadyExists, "Example IO Error."))
    }

    pub fn testingconvert() -> ParseResult<()> {
        hb_parse::error::ConvertInto::<ParseResult<()>>::convert(Err(std::io::Error::new(std::io::ErrorKind::AlreadyExists, "Example IO Error.")))?;
        Ok(())
    }

    pub fn ParseError() -> ParseResult<()> {
        Err(ParseError::with_msg("Example ParseError."))
    }
}

fn main() {
    println!("{}", Example::basic_parseerror().err().unwrap());
    println!("{}", Example::returned_parseerror().err().unwrap());
    println!("{:?}", Example::question_parseerror().err().unwrap());
    println!("{:?}", Example::basic_ioerror().err().unwrap());
    println!("{}", Example::returned_ioerror().err().unwrap());
    println!("{:?}", Example::question_ioerror().err().unwrap());
    println!("{:?}", Example::testingconvert().err().unwrap());
}