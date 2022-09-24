use crate::SourceResult;

/// Interface for different implementations of sources of data for the parser.
/// Provides various function which will allow the parsing of data without having
/// multiple buffers.
pub trait Source {
    /// Gets the next char and moves the pointer
    fn next(&mut self) -> SourceResult<Option<(usize, char)>>;
    /// Gets the next char but does not move the pointer
    fn peek(&mut self) -> SourceResult<Option<(usize, char)>>;
    /// Moves the pointer backwards
    fn move_back(&mut self, n: usize) -> SourceResult<()>;
    /// Moves the pointer forwards
    fn move_forward(&mut self, n: usize) -> SourceResult<()>;
    /// Consumes the chars at the beginning moving the window
    /// forward without returning any of the chars
    fn consume(&mut self, n: usize) -> SourceResult<()>;
    /// Extracts the chars at the beginning of the window
    /// moving the window forward and returning the chars
    fn extract(&mut self, n: usize) -> SourceResult<String>;
    /// Reads a sub-string from the window, without affecting
    /// any pointers
    fn read_substr(&mut self, start: usize, n: usize) -> SourceResult<String>;
    /// Get the current pointer location
    fn get_pointer_loc(&self) -> usize;
    /// Resets the pointer to the start of the window
    fn reset_pointer_loc(&mut self);
    /// Gets up to 80 chars around the current pointer
    fn get_context(&self) -> String;
}
