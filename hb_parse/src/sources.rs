pub trait Source {
    fn next(&mut self) -> Option<char>;
}

pub struct StrSource<'a> {
    s: &'a str,
    iter: std::str::Chars,
}

impl StrSource<'a> {
    fn new(s: &'a str) -> StrSource {
        StrSource {
            s: s,
            iter: s.chars(),
        }
    }
}

impl Source for StrSource {
    fn next(&mut self) -> Option<char> {
        self.iter.next()
    }
}
