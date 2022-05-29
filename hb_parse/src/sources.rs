pub trait Source {
    fn next(&mut self) -> Option<char>;
    fn peek(&mut self) -> Option<char>;
}

pub struct StrSource<'a> {
    s: &'a str,
    iter: std::str::Chars<'a>,
    next: Option<char>,
}

impl<'a> StrSource<'a> {
    pub fn new(s: &'a str) -> StrSource<'a> {
        let mut s_iter = s.chars();
        let next = s_iter.next();
        StrSource {
            s: s,
            iter: s_iter,
            next: next,
        }
    }
}

impl Source for StrSource<'_> {
    fn next(&mut self) -> Option<char> {
        match self.next {
            None => return None,
            Some(c) => {
                self.next = self.iter.next();
                return Some(c);
            }
        }
    }
    fn peek(&mut self) -> Option<char> {
        self.next
    }
}
