use hb_error::*;
use hb_parse::error::{ParseError, ParseResult};
struct Example;

impl Example {
    #[context("error encounted while basic_parseerror")]
    fn basic_parseerror() -> ParseResult<()> {
        Example::parse_error()
    }

    #[context("error encounted while returned_parseerror")]
    fn returned_parseerror() -> ParseResult<()> {
        return Example::parse_error();
    }

    #[context("error encounted while question_parseerror")]
    fn question_parseerror() -> ParseResult<()> {
        Example::parse_error()?;
        Ok(())
    }

    #[context("error encounted while basic_ioerror")]
    fn basic_ioerror() -> ParseResult<()> {
        Ok(Example::ioerror()?)
    }

    #[context("error encounted while returned_ioerror")]
    fn returned_ioerror() -> ParseResult<()> {
        return Example::ioerror();
    }

    #[context("error encounted while question_ioerror")]
    fn question_ioerror() -> ParseResult<()> {
        Example::ioerror()?;
        Ok(())
    }

    #[convert_error]
    fn convert_ioerror() -> ParseResult<()> {
        Ok(Example::ioerror()?)
    }

    fn ioerror() -> std::io::Result<()> {
        Err(std::io::Error::new(
            std::io::ErrorKind::AlreadyExists,
            "Example IO Error.",
        ))
    }

    fn parse_error() -> ParseResult<()> {
        Err(ParseError::with_msg("Example ParseError."))
    }
}

fn main() {
    println!(
        "Basic Example:\n{}\n",
        Example::basic_parseerror().err().unwrap()
    );
    println!(
        "Returned Example:\n{}\n",
        Example::returned_parseerror().err().unwrap()
    );
    println!(
        "QuestionMark Example:\n{}\n",
        Example::question_parseerror().err().unwrap()
    );
    println!(
        "Basic Example:\n{}\n",
        Example::basic_ioerror().err().unwrap()
    );
    println!(
        "Returned Example:\n{}\n",
        Example::returned_ioerror().err().unwrap()
    );
    println!(
        "QuestionMark Example:\n{}\n",
        Example::question_ioerror().err().unwrap()
    );
    println!(
        "Convert Example:\n{}\n",
        Example::convert_ioerror().err().unwrap()
    )
}
