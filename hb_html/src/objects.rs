use crate::error::ParseHtmlError;
use crate::parsing::{parse_css_selector_rule, parse_html_tag, ParsedTagType};
use crate::querying::{HtmlQuery, HtmlQueryable};
use std::collections::HashMap;
use std::fmt;
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

impl fmt::Display for HtmlTag {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "<Tag: {} , IDs: {:?}, Classes: {:?}, Attributes: {:?}, Contents: {:?}>",
            self.tag, self.ids, self.classes, self.attributes, self.contents,
        )
    }
}

impl HtmlTag {
    /// Converts the HtmlTag into a string formatted as HTML.
    fn to_html_string(&self) -> String {
        let mut res = format!("<{}", self.tag);
        if self.ids.len() > 0 {
            res.push_str(" id=\"");
            res.push_str(self.ids.join(" ").as_str());
            res.push_str("\"")
        }
        if self.classes.len() > 0 {
            res.push_str(" class=\"");
            res.push_str(self.classes.join(" ").as_str());
            res.push_str("\"")
        }
        if self.attributes.len() > 0 {
            for attr in self.attributes.keys() {
                res.push_str(format!(" {}=\"{}\"", attr, self.attributes[attr]).as_str());
            }
        }
        res.push_str(">");
        if self.contents.len() > 0 {
            for content in &self.contents {
                res.push_str(content.to_html_string().as_str())
            }
        }
        res.push_str(format!("</{}>", self.tag).as_str());
        res
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

impl HtmlQueryable for Vec<HtmlNode> {
    fn query(&self) -> HtmlQuery {
        HtmlQuery::new(self)
    }
}

impl HtmlNode {
    /// Converts the HtmlNode into a string formatted as HTML.
    fn to_html_string(&self) -> String {
        match &self {
            HtmlNode::Comment(c) => format!("<!-- {} --!>", c),
            HtmlNode::Tag(t) => t.to_html_string(),
            HtmlNode::Text(t) => t.to_string(),
        }
    }
}

#[cfg(test)]
mod html_tag_tests {
    use super::*;

    #[test]
    fn html_tag_to_html_string() {
        let tests = vec![
            "<div></div>",
            "<div class=\"c1\"></div>",
            "<div id=\"i1\"></div>",
            "<div id=\"i1\" class=\"c1\"></div>",
            "<div other=\"o1\"></div>",
            "<div id=\"i1\" class=\"c1\" other=\"o1\"></div>",
            "<div id=\"i1\" class=\"c1\" other=\"o1\">text</div>",
            "<div id=\"i1\" class=\"c1\" other=\"o1\">text<p>more text</p></div>",
        ];
        for test in &tests {
            assert_eq!(
                test.parse::<HtmlTag>().unwrap().to_html_string().as_str(),
                *test
            );
        }
    }
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
    pub fn find(&self, selector: &str) -> HtmlQuery {
        let mut query = self.query();
        query.find(selector);
        query
    }
}

impl HtmlQueryable for HtmlDocument {
    /// Creates a new [`HtmlQuery`] from this [`HtmlDocument`]
    fn query(&self) -> HtmlQuery {
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

//Example complex selector...
// head div > div#titleblock p.bold#title:first-of-type
// We want to match right to left...
// 1. does the current node match p.bold#title:first-of-type
// 2. does it have an ancestor which is
// ...
//
// This should be done using a iterator backwards over the path,
// and checking over the list of CSS relations, moving to the next
// when it finds a node that matches (or exiting early if the
// specific node did not match - eg parent/sibling)

#[derive(Debug, PartialEq)]
/// Represents the relationship that is next to be matched in the list of selector items.
pub enum CssSelectorRelationship {
    Parent(CssSelectorItem),
    Ancestor(CssSelectorItem),
    PreviousSibling(CssSelectorItem),
    PreviousSiblingOnce(CssSelectorItem),
    Current(CssSelectorItem),
}

#[derive(Debug, PartialEq)]
/// represents all of the CSS selectors which follow a :, for example :last-child
pub enum CssRefiner {
    Checked,
    Default,
    Disabled,
    Enabled,
    Optional,
    Required,
    ReadOnly,
    ReadWrite,
    Empty,
    FirstChild,
    LastChild,
    NthChild(CssRefinerNumberType),
    NthLastChild(CssRefinerNumberType),
    OnlyChild,
    FirstOfType,
    LastOfType,
    NthOfType(CssRefinerNumberType),
    NthLastOfType(CssRefinerNumberType),
    OnlyOfType,
    Not(CssSelector),
    Root,
}

#[derive(Debug, PartialEq)]
/// Used for CSS Selectors such as :nth-child(x) where x can be odd, even or a specific number
pub enum CssRefinerNumberType {
    Odd,
    Even,
    Specific(usize),
    Functional((i32, i32)),
}

#[derive(Debug, PartialEq)]
/// Used to represents the different types of attributes selections for example [attribute=value]
pub enum CssAttributeCompareType {
    /// [attribute]
    Present(String),
    /// [attribute=value]
    Equals((String, String)),
    /// [attribute|=value]
    EqualsOrBeingsWith((String, String)),
    /// [attribute^=value]
    BeginsWith((String, String)),
    /// [attribute$=value]
    EndsWith((String, String)),
    /// [attribute*=value]
    Contains((String, String)),
    /// [attribute~=value]
    ContainsWord((String, String)),
}

#[derive(Debug, PartialEq)]
/// Represents a CSS selector for a particular node
pub struct CssSelectorItem {
    pub tag: Option<String>,
    pub classes: Option<Vec<String>>,
    pub ids: Option<Vec<String>>,
    pub refiners: Option<Vec<CssRefiner>>, // anything like :... eg :only-child
    pub attributes: Option<Vec<CssAttributeCompareType>>,
}

impl CssSelectorItem {
    pub fn new() -> CssSelectorItem {
        CssSelectorItem {
            tag: None,
            classes: None,
            ids: None,
            refiners: None,
            attributes: None,
        }
    }
}

#[derive(Debug, PartialEq)]
/// Represents a rule that must match for a CSS selector
pub struct CssSelectorRule {
    pub rules: Vec<CssSelectorRelationship>,
}

impl CssSelectorRule {
    pub fn new() -> CssSelectorRule {
        CssSelectorRule { rules: vec![] }
    }
}

#[derive(Debug, PartialEq)]
/// Represents a CSS selector which could be anything (*) or based on a some selection rules.
/// CSS selectors all multiple different match rules seperated by a comma. This is handle by
/// having each matching rule in a vector.
pub enum CssSelector {
    Any,
    Specific(Vec<CssSelectorRule>),
}

impl FromStr for CssSelector {
    type Err = ParseHtmlError;
    fn from_str(selector: &str) -> Result<Self, <Self as std::str::FromStr>::Err> {
        if selector == "*" {
            return Ok(CssSelector::Any);
        }
        let mut rules: Vec<CssSelectorRule> = vec![];
        for s in selector.split(",").map(|x| x.trim()) {
            //parse rule and add to rules;
            rules.push(parse_css_selector_rule(s)?);
        }
        if rules.len() > 0 {
            return Ok(CssSelector::Specific(rules));
        }
        Err(ParseHtmlError::with_msg(format!(
            "No valid CSS selector found in {}",
            selector
        )))
    }
}
