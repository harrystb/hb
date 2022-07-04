use crate::objects::HtmlDocument;
use hb_parse::error::ParseResult;
use hb_parse::source::Source;
use hb_parse::CommonParserFunctions;

pub trait HtmlParserFunctions {
    fn parse_html(&mut self) -> ParseResult<HtmlDocument>;
}

impl<S: Source> HtmlParserFunctions for S {
    fn parse_html(&mut self) -> ParseResult<HtmlDocument> {
        todo!();
    }
}

trait HtmlParserInnerFunctions {
    fn parse_doctype(&mut self) -> ParseResult<String>;
}

impl<S: Source> HtmlParserInnerFunctions for S {
    fn parse_doctype(&mut self) -> ParseResult<String> {
        todo!();
    }
}
