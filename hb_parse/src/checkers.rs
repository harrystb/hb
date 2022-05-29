pub trait ParseChecker {
    fn parse(&mut self, c: char) -> bool;
}
pub struct CharChecker {
    ending: char,
}

impl CharChecker {
    pub fn new(ending: char) -> CharChecker {
        CharChecker { ending: ending }
    }
}

impl<'a> ParseChecker for CharChecker {
    fn parse(&mut self, c: char) -> bool {
        if c == self.ending {
            return true;
        }
        false
    }
}
