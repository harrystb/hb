use crate::error::ParseError;

trait ParseChecker {
    fn parse(&mut self, c: char) -> Result<Option<(usize, usize)>, ParseError>;
}
struct CharParser {
    start_index: usize,
    chars_checked: usize,
    ending: char,
}

impl CharParser {
    fn new(ending: char, start_index: usize) -> CharParser {
        CharParser {
            start_index: start_index,
            chars_checked: 0,
            ending: ending,
        }
    }
}

impl<'a> ParseChecker for CharParser {
    fn parse(&mut self, c: char) -> Result<Option<(usize, usize)>, ParseError> {
        if c == self.ending {
            return Ok(Some((
                self.start_index,
                self.start_index + self.chars_checked,
            )));
        } else {
            self.chars_checked += 1;
            return Ok(None);
        }
    }
}
