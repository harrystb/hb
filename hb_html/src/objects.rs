use crate::error::ParseHtmlError;
use crate::parsing::{parse_html_tag, ParsedTagType};
use std::collections::HashMap;
use std::str::FromStr;

/// Represents a HTML Tag including both attributes and contents.
/// Contents can include HTML comments, HTML tags and text.
/// The contents is stored as a [`HtmlNode`].
///
/// # Example:
///
/// This HTML tag
/// ```text
/// <div class="heading" id=top other="3">This is an example</div>
/// ```
///
/// would be represented by
///
/// ```ignore
/// HtmlTag{
///    tag : "div",
///    ids : ["top"],
///    classes : ["heading"],
///    attributes : {"other" : "3"},
///    contents : [HtmlNode::Text("This is an example")],
/// }
/// ```
#[derive(Debug, Clone)]
pub struct HtmlTag {
    /// HTML tag name
    pub tag: String,
    /// List of Ids from the attribute id="" in the HTML tag
    pub ids: Vec<String>,
    /// List of Classes from the attribute class="" in the HTML tag
    pub classes: Vec<String>,
    /// All other attributes from the HTML tag.
    pub attributes: HashMap<String, String>,
    /// The contents of the HTML tag, stores as [`HtmlNode`] objects.
    pub contents: Vec<HtmlNode>,
}

impl PartialEq for HtmlTag {
    fn eq(&self, other: &HtmlTag) -> bool {
        if self.tag != other.tag {
            return false;
        }
        if self.ids.len() != other.ids.len() {
            return false;
        }
        for id in &self.ids {
            if !other.ids.contains(id) {
                return false;
            }
        }
        if self.classes.len() != other.classes.len() {
            return false;
        }
        for class in &self.classes {
            if !other.classes.contains(class) {
                return false;
            }
        }
        if self.attributes != other.attributes {
            return false;
        }
        return true;
    }
}

impl HtmlTag {
    /// Create an empty [`HtmlTag`] with the tag name provided.
    ///
    /// # Arguments
    ///
    /// * `tag` - The tag name. Can be anything that implements Into\<String\>
    pub fn new<T: Into<String>>(tag: T) -> HtmlTag {
        HtmlTag {
            tag: tag.into(),
            ids: vec![],
            classes: vec![],
            contents: vec![],
            attributes: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
/// Represents the different types of content that can be found inside HTML
/// tags.
pub enum HtmlNode {
    /// A HTML tag stored as a [`HtmlTag`].
    Tag(HtmlTag),
    /// A HTML comment such as \<!-- This is a comment --!\>
    Comment(String),
    /// Text content.
    Text(String),
}

#[derive(Debug, Clone)]
/// Represents a whole HTML document.
///
/// # Example
///
/// This is a basic HTML document
/// ```text
/// <!DOCTYPE html>
/// <!-- An example HTML Document -->
/// <html><head>
/// <title>A HTML Document (Test File)</title>
/// </head>
/// <body>
/// <h1 class=heading>A HTML Document (Test File)</h1>
/// <p>A blank HTML document.</p>
/// </body></html>"
/// ```
///
/// This be represented by
///
/// ```text
/// HtmlDocument {
/// doctype : "html",
/// nodes : [
///    HtmlNode::Comment("An example HTML Document"),
///    HtmlNode::Tag(HtmlTag { tag : "html" ... })
///    ] ,
/// }
/// ```
pub struct HtmlDocument {
    /// The doctype string from the document, usually "html".
    pub doctype: String,
    /// Representation of all HTML tags, comments or text that appears at the
    /// top level in the document.
    pub nodes: Vec<HtmlNode>,
}

impl HtmlDocument {
    /// Creates a empty [`HtmlDocument`].
    pub fn new() -> HtmlDocument {
        let v: Vec<HtmlNode> = vec![];
        HtmlDocument {
            doctype: String::new(),
            nodes: v,
        }
    }
}

impl FromStr for HtmlDocument {
    type Err = ParseHtmlError;
    fn from_str(html_str: &str) -> Result<Self, <Self as std::str::FromStr>::Err> {
        let mut doc = HtmlDocument::new();
        let mut chs = html_str.chars();
        let mut buffer = String::new();
        while let Some(ch) = chs.next() {
            if ch == '<' {
                if buffer.len() > 0 {
                    doc.nodes.push(HtmlNode::Text(buffer));
                    buffer = String::new();
                }
                match parse_html_tag(&mut chs)? {
                    ParsedTagType::EndTag(t) => {
                        return Err(ParseHtmlError::new(format!(
                            "Found end tag {} before start tag.",
                            t
                        )))
                    }
                    ParsedTagType::NewTag(tag) => doc.nodes.push(HtmlNode::Tag(tag)),
                    ParsedTagType::Comment(c) => doc.nodes.push(HtmlNode::Comment(c)),
                    ParsedTagType::DocType(doctype) => {
                        if doc.doctype.len() > 0 {
                            //it was already defined..
                            return Err(ParseHtmlError::new(format!(
                                "Doctype was defined twice, first {} and second {}",
                                doc.doctype, doctype,
                            )));
                        }
                        doc.doctype = doctype;
                    }
                }
            }
        }
        Ok(doc)
    }
}
