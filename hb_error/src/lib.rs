//! This crate defines from error related traits and generic implementations
//! used by other hb projects.
//! # ConvertFrom and ConvertTo
//! The [ConvertFrom] and [ConvertInto] traits as well as the default implementations
//! are used to allow easy handling of Result objects. See hb_macro for use cases.
//! ## Example
//! ```
//! use hb_parse::error::ParseError;
//! use hb_err::ConvertInto;
//! // Convert from IO Error into a ParseError as ParseError implements the From<std::io::Error> trait
//! let er: Result<(),ParseError> = Result<(),std::io::Error>::Err(std::io::Error::new(std::io::ErrorKind::AlreadyExists, "Example IO Error.")).convert();
//! ```

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
