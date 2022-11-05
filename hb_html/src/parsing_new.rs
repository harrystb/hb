use crate::objects::HtmlDocument;
use hb_error::{context, ErrorContext};
use hb_parse::error::{ParseError, ParseResult};
use hb_parse::source::Source;
use hb_parse::CommonParserFunctions;

pub trait HtmlParserFunctions {
    fn parse_html(&mut self) -> ParseResult<HtmlDocument>;
}

impl<S: Source> HtmlParserFunctions for S {
    fn parse_html(&mut self) -> ParseResult<HtmlDocument> {
        let mut doc = HtmlDocument::new();
        doc.doctype = self
            .parse_doctype()
            .map_err(|e| e.make_inner().msg("could not parse the html document"))?;
        todo!();
    }
}

trait HtmlParserInnerFunctions {
    fn parse_doctype(&mut self) -> ParseResult<String>;
}

impl<S: Source> HtmlParserInnerFunctions for S {
    #[context("could not parse doctype")]
    fn parse_doctype(&mut self) -> ParseResult<String> {
        if self.get_pointer_loc() != 0 {
            return Err(ParseError::new().msg(format!("Parser has already been used, and has left a pointer at position {} (which should be 0).", self.get_pointer_loc())));
        }
        self.skip_whitespace()?;
        match self.next()? {
            None => {
                return Err(ParseError::new()
                    .msg("could not parse doctype as there are no characters left"))
            }
            Some((_, c)) => {
                if c != '<' {
                    self.reset_pointer_loc();
                    return Err(ParseError::new()
                        .msg(format!(
                        "could not parse doctype because the next character is {} rather than '<'\n{}",
                        c, self.get_context())));
                }
            }
        }

        if self.read_symbol().map_err(|e| {
            e.make_inner()
                .msg(format!("didn't find a '!'\n{}", self.get_context()))
        })? != '!'
        {
            return Err(ParseError::new().msg(format!(
                "could not parse doctype because the next character is not '!'\n{}",
                self.get_context(),
            )));
        }

        if self.read_word()?.to_uppercase() != "DOCTYPE" {
            return Err(ParseError::new().msg(format!(
                "could not parse doctype because word found is not 'DOCTYPE'\n{}",
                self.get_context()
            )));
        }
        self.skip_whitespace()?;
        let start_i = self.get_pointer_loc();
        while let Some((i, c)) = self.next()? {
            if c == '>' {
                let doctype = self.read_substr(start_i, self.get_pointer_loc() - start_i - 1)?;
                self.consume(self.get_pointer_loc())?;
                return Ok(doctype);
            }
        }
        return Err(ParseError::new().msg(format!(
            "could not parse doctype because the '>' could not be found\n{}",
            self.get_context()
        )));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hb_parse::StrParser;
    #[test]
    fn parse_doctype_test() {
        let mut source = StrParser::new(" <!DOCTYPE Something?>");
        assert_eq!(source.parse_doctype().unwrap(), "Something?".to_owned());
        let mut source = StrParser::new(" <DOCTYPE Something?");
        assert_eq!(
            format!("{}", source.parse_doctype().err().unwrap()),
            format!(
                "{}",
                ParseError::new()
                    .msg("cound not parse symbol because 'D' is not classified as a symbol")
                    .make_inner()
                    .msg("didn't find a '!'\n<DOCTYPE Something?\n  ^\n")
                    .make_inner()
                    .msg("could not parse doctype")
            )
        );
        let mut source = StrParser::new(" <!DOCTYPE Something?");
        assert_eq!(format!("{}", source.parse_doctype().err().unwrap()), "");
    }
}
