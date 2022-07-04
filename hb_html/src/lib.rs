//! Crate to parse and extract data from HTML documents.
//! # Example
//!
//! ```
//! use hb_html::objects::HtmlDocument;
//! use hb_html::querying::HtmlQuery;
//! let html_str = r#"<!DOCTYPE html>
//! <!-- An example HTML Document -->
//! <html><head>
//! <title>A HTML Document (Test File)</title>
//! </head>
//! <body>
//! <h1 class=heading>A HTML Document (Test File)</h1>
//! <p>A blank HTML document.</p>
//! </body></html>"#;
//! let html_doc = match html_str.parse::<HtmlDocument>() {
//!     Ok(d) => d,
//!     Err(_) => return (),
//! };
//! let query = html_doc.new_query();
//! query.find_with_tag("div").find_with_tag("p");
//! ```

pub mod error;
pub mod objects;
mod parsing;
mod parsing_new;
pub mod querying;
