//! This crate defines from error related traits and generic implementations
//! used by other hb projects.
//! # ConvertFrom and ConvertTo
//! The [ConvertFrom] and [ConvertInto] traits as well as the default implementations
//! are used to allow easy handling of Result objects. See hb_macro for use cases.
//! ## Example
//! ```
//! use hb_error::*;
//!
//! #[hberror]
//! struct ExampleError {
//!     #[Source]
//!     IoError: std::io::Error,
//! }
//! // Convert from IO Error into a ParseError as ParseError implements the From<std::io::Error> trait
//! let er: Result<(),ExampleError> = Result::<(),std::io::Error>::Err(std::io::Error::new(std::io::ErrorKind::AlreadyExists, "Example IO Error.")).convert();
//! ```
//! # Context and Context Documentation
//! the [context] and [context_doc] macros allow a simplified way to add context into errors that are
//! generated within the functions. The example below shows how simple it is to set up a error type
//! that can support auto-conversion and adding context via the [context] and [context_doc] macros.
//! See [context], [context_doc] and [hberror] for more information on what that macro scaffolds for
//! the error types.
//!
//! ```
//! use hb_error::*;
//!
//! #[hberror]
//! struct ExampleError {
//!     #[Source]
//!     IoError: std::io::Error,
//! }
//!
//! //Some functions to generate errors for the examples
//! fn io_error() -> std::io::Result<()> {
//! Err(std::io::Error::new(
//! std::io::ErrorKind::AlreadyExists,
//! "Example IO Error.",
//! ))
//! }
//!
//! fn example_error() -> Result<(), ExampleError> {
//! Err(ExampleError::new().msg("Generated ExampleError."))
//! }
//!
//! // Example
//! // Add context onto a fall through Err object of type ExampleError
//! // The fall through Err before the macro adds context will be:
//! // ExampleError {
//! //   msg: "Generated ExampleError.",
//! //   inner_error: vec![]
//! // }
//! //
//! // The err returned from this function (ie after the context macro changes it) will be:
//! // ExampleError {
//! //   msg: "additional context - basic example",
//! //   inner_error: vec!["Generated ExampleError."]
//! // }
//! #[context("addition context - basic example")]
//! fn basic_exampleerror() -> Result<(), ExampleError> {
//! example_error()
//! }
//!
//! // Example
//! // Convert from an io::Error into a ExampleError and add context
//! // The io::Error before conversion will be
//! // std::io::Error::new(
//! //     std::io::ErrorKind::AlreadyExists,
//! //     "Example IO Error.",
//! // )
//! //
//! // after conversion but before the context is added the ExampleError will be:
//! // ExampleError {
//! //   msg: "encountered an IO Error",
//! //   inner_error: vec![]
//! // }
//! //
//! // The final error will be:
//! // ExampleError {
//! //   msg: "additional context - basic example" ,
//! //   inner_error: vec!["encountered an IO Error"]
//! // }
//! #[context("addition context - basic example")]
//! fn basic5_exampleerror() -> Result<(), ExampleError> {
//! return io_error();
//! }
//! ```
//! See examples\error_example.rs for more examples.

pub use hb_macros::*;
mod convert;
pub use convert::*;
mod context;
pub use context::*;

// TODO when documenting features comes to stable rust then add appropriate flags so the
// example docs are displayed.
//#[hberror("{self.code}: {self.msg}{self.inner_msgs.join(\"\n...because...\")}")]
//pub struct exampleerror {
//#[default(0)]
//code: i32,
//#[source]
//ioerror: std::io::error,
//}
