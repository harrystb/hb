use crate::error::ParseHtmlError;
use crate::parsing::{parse_html_tag, ParsedTagType};
use crate::querying::HtmlQuery;
use std::collections::HashMap;
use std::str::FromStr;

/// Represents a HTML Tag including both attributes and contents.
/// Contents can include HTML comments, HTML tags and text.
/// The contents is stored as a [`HtmlNode`].
///
/// # Example:
///
/// Single level HTML tag
///
/// ```
/// use hb_html::objects::{HtmlTag, HtmlNode};
/// //Parse from a str
/// let html_txt = r#"<div class="border" id=check>Inner Fire</div>"#;
/// let html_tag = html_txt.parse::<HtmlTag>().unwrap();
/// //Manually create the tag
/// let mut manual_tag = HtmlTag::new("div");
/// manual_tag.classes.push("border".to_owned());
/// manual_tag.ids.push("check".to_owned());
/// manual_tag.contents.push(HtmlNode::Text("Inner Fire".to_owned()));
/// //They are equivalent
/// assert_eq!(html_tag, manual_tag);
/// ```
/// Multi level HTML tag
///
/// ```
/// use hb_html::objects::{HtmlTag, HtmlNode};
/// //Parse from a str
/// let html_txt = r#"<div class="border" id=check>Inner <div class="wavy">Fire</div></div>"#;
/// let html_tag = html_txt.parse::<HtmlTag>().unwrap();
/// //Manually create the tag
/// let mut manual_tag = HtmlTag::new("div");
/// manual_tag.classes.push("border".to_owned());
/// manual_tag.ids.push("check".to_owned());
/// manual_tag.contents.push(HtmlNode::Text("Inner ".to_owned()));
/// let mut manual_tag_inner = HtmlTag::new("div");
/// manual_tag_inner.classes.push("wavy".to_owned());
/// manual_tag_inner.contents.push(HtmlNode::Text("Fire".to_owned()));
/// manual_tag.contents.push(HtmlNode::Tag(manual_tag_inner));
/// //They are equivalent
/// assert_eq!(html_tag, manual_tag);
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
impl FromStr for HtmlTag {
    type Err = ParseHtmlError;
    fn from_str(html_str: &str) -> Result<Self, <Self as std::str::FromStr>::Err> {
        let mut res_tag = None;
        let mut chs = html_str.chars();
        let mut buffer = String::new();
        while let Some(ch) = chs.next() {
            if ch == '<' {
                if buffer.len() > 0 {
                    return Err(ParseHtmlError::new(format!(
                        "Found text {} before start of the tag.",
                        buffer
                    )));
                }
                match parse_html_tag(&mut chs)? {
                    ParsedTagType::EndTag(t) => {
                        return Err(ParseHtmlError::new(format!(
                            "Found end tag {} before start tag.",
                            t
                        )))
                    }
                    ParsedTagType::NewTag(tag) => match res_tag {
                        None => {
                            res_tag = Some(tag);
                        }
                        Some(t) => {
                            return Err(ParseHtmlError::new(format!(
                                "Found second tag {:?} after the first tag {:?}.",
                                tag, t
                            )))
                        }
                    },
                    ParsedTagType::Comment(c) => {
                        return Err(ParseHtmlError::new(format!(
                            "Found HTML comment {} before a tag",
                            c
                        )))
                    }
                    ParsedTagType::DocType(doctype) => {
                        return Err(ParseHtmlError::new(format!(
                            "Found HTML DOCTYPE tag {} before a standard HTML tag",
                            doctype
                        )))
                    }
                }
            }
        }
        match res_tag {
            None => return Err(ParseHtmlError::new("No tag found.".to_owned())),
            Some(t) => Ok(t),
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
/// ```
/// use hb_html::objects::HtmlDocument;
/// let html_str = r#"<!DOCTYPE html>
/// <!-- An example HTML Document -->
/// <html><head>
/// <title>A HTML Document (Test File)</title>
/// </head>
/// <body>
/// <h1 class=heading>A HTML Document (Test File)</h1>
/// <p>A blank HTML document.</p>
/// </body></html>"#;
/// let html_doc = match html_str.parse::<HtmlDocument>() {
///     Ok(d) => d,
///     Err(_) => return (),
/// };
/// ```
pub struct HtmlDocument {
    /// The doctype string from the document, usually "html".
    pub doctype: String,
    /// All HTML tags, comments or text that appears at the top level in the document.
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

    pub fn new_query(&self) -> HtmlQuery {
        HtmlQuery::new(&self.nodes)
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
