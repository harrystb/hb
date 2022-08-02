/// The ErrorContext trait should be implemented for the context macro to work.
pub trait ErrorContext {
    fn make_inner(self) -> Self;
    fn msg<T: Into<String>>(self, msg: T) -> Self;
}
