/// The ErrorContext trait should be implemented for the context macro to work.
pub trait ErrorContext {
    fn make_inner(self) -> Self;
    fn msg<T: Into<String>>(self, msg: T) -> Self;
}
impl<O, E: ErrorContext> ErrorContext for Result<O, E> {
    fn make_inner(self) -> Self {
        self.map_err(|er| er.make_inner())
    }
    fn msg<T: Into<String>>(self, msg: T) -> Self {
        self.map_err(|er| er.msg(msg))
    }
}
