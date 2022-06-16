use crate::error::{ParseError, ParseResult};

/// Interface for different implementations of sources of data for the parser.
/// Provides various function which will allow the parsing of data without having
/// multiple buffers.
pub trait Source {
    /// Gets the next char and moves the pointer
    fn next(&mut self) -> ParseResult<Option<(usize, char)>>;
    /// Gets the next char but does not move the pointer
    fn peek(&mut self) -> ParseResult<Option<(usize, char)>>;
    /// Moves the pointer backwards
    fn move_back(&mut self, n: usize) -> ParseResult<()>;
    /// Moves the pointer forwards
    fn move_forward(&mut self, n: usize) -> ParseResult<()>;
    /// Consumes the chars at the begining moving the window
    /// forward without returning any of the chars
    fn consume(&mut self, n: usize) -> ParseResult<()>;
    /// Extracts the chars at the begining of the window
    /// moving the window forward and returning the chars
    fn extract(&mut self, n: usize) -> ParseResult<String>;
    /// Reads a sub-string from the window, without affecting
    /// any pointers
    fn read_substr(&mut self, start: usize, n: usize) -> ParseResult<String>;
    /// Get the current pointer location
    fn get_pointer_loc(&mut self) -> usize;
    /// Resets the pointer to the start of the window
    fn reset_pointer_loc(&mut self);
}
