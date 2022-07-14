use hb_macros::context;
use hb_parse::error::{ParseResult, ParseError};
struct eh;

impl eh {
    #[context("Something")]
    pub fn eh() -> ParseResult<()> {
        eh::b()?;
        Ok(())
    }

    pub fn b() -> ParseResult<()> {
        Err(ParseError::with_msg("inner"))
    }
}

fn main() {
    println!("{}", eh::eh().err().unwrap())
}