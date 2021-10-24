//! Crate to parse and extract data from HTML documents.

mod error;
use error::{HtmlDocError, HtmlMatchError, ParseHtmlError};
use std::collections::{HashMap, VecDeque};
use std::str::FromStr;

#[derive(Debug, Clone)]
/// Represents a HTML Tag including both attributes and contents.
/// Contents can include HTML comments, HTML tags and text.
/// The contents is stored as a [`HtmlNode`].
///
/// # Example:
///
/// This HTML tag
/// ```
/// <div class="heading" id=top other="3">This is an example</div>
/// ```
///
/// would be represented by
///
/// ```
/// HtmlTag{
///    tag : "div",
///    ids : ["top"],
///    classes : ["heading"],
///    attributes : {"other" : "3"},
///    contents : [HtmlNode::Text("This is an example")],
/// }
/// ```
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
/// ```
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
/// ```
/// HtmlDocument {
/// doctype : "html",
/// nodes : [
///    HtmlNode::Comment("An example HTML Document"),
///    HtmlNode::Tag(HtmlTag { tag : "html" ... })
///    ] ,
/// }
/// ```
struct HtmlDocument {
    /// The doctype string from the document, usually "html".
    doctype: String,
    /// Representation of all HTML tags, comments or text that appears at the
    /// top level in the document.
    nodes: Vec<HtmlNode>,
}

impl HtmlDocument {
    /// Creates a empty [`HtmlDocument`].
    fn new() -> HtmlDocument {
        let v: Vec<HtmlNode> = vec![];
        HtmlDocument {
            doctype: String::new(),
            nodes: v,
        }
    }
}

// Read from the iterator until a quoted string or word is found (ignoring leading whitespace) then return the string and the character that ended the string
// Endings of a single word can be whitespace or >
fn parse_string(chs: &mut std::str::Chars) -> Result<(String, char), ParseHtmlError> {
    let mut buffer = String::with_capacity(50);
    let mut is_quoted = false;
    //consume leading whitespace
    let ch = get_next_non_whitespace(chs)?;
    //check if first char is a "
    if ch == '"' {
        is_quoted = true;
    } else {
        buffer.push(ch);
    }
    while let Some(ch) = chs.next() {
        if is_quoted {
            if ch == '"' {
                return Ok((buffer, ch));
            }
        } else {
            if ch.is_ascii_whitespace() || ch == '>' {
                return Ok((buffer, ch));
            }
        }
        buffer.push(ch);
    }
    return Err(ParseHtmlError::new(format!(
        "End of string '{}' not found",
        buffer
    )));
}

fn parse_until_one_of(
    chs: &mut std::str::Chars,
    end_chars: Vec<char>,
    include_ending: bool,
) -> Result<String, ParseHtmlError> {
    let mut buffer = String::with_capacity(50);
    while let Some(ch) = chs.next() {
        for &end_char in &end_chars {
            if ch == end_char {
                if include_ending {
                    buffer.push(ch);
                }
                return Ok(buffer);
            }
        }
        buffer.push(ch);
    }
    return Err(ParseHtmlError::new(format!(
        "End of string '{}' encountered before any end char '{:?}' was found",
        buffer, end_chars
    )));
}

fn parse_until_char(
    chs: &mut std::str::Chars,
    end_char: char,
    include_ending: bool,
) -> Result<String, ParseHtmlError> {
    let mut buffer = String::with_capacity(50);
    while let Some(ch) = chs.next() {
        if ch == end_char {
            if include_ending {
                buffer.push(ch);
            }
            return Ok(buffer);
        }
        buffer.push(ch);
    }
    return Err(ParseHtmlError::new(format!(
        "End of string '{}' encountered before end char '{}' was found",
        buffer, end_char
    )));
}

fn parse_until_str(
    chs: &mut std::str::Chars,
    end_str: &str,
    include_ending: bool,
) -> Result<String, ParseHtmlError> {
    let mut ending_buffer = VecDeque::with_capacity(end_str.len());
    let mut buffer = String::with_capacity(50);
    let end_chars = end_str.chars().collect::<Vec<char>>();
    while let Some(ch) = chs.next() {
        if ch == end_chars[ending_buffer.len()] {
            ending_buffer.push_front(ch);
            if ending_buffer.len() == end_str.len() {
                if include_ending {
                    while ending_buffer.len() > 0 {
                        buffer.push(ending_buffer.pop_back().unwrap());
                    }
                }
                return Ok(buffer);
            }
        } else {
            //flush ending_buffer into the buffer as we no longer
            while ending_buffer.len() > 0 {
                buffer.push(ending_buffer.pop_back().unwrap());
            }
            buffer.push(ch);
        }
    }
    return Err(ParseHtmlError::new(format!(
        "End of string '{}{}' encountered before any end string '{}' was found",
        buffer,
        ending_buffer.iter().collect::<String>(),
        end_str
    )));
}

fn parse_until<F: Fn(&char) -> bool>(
    chs: &mut std::str::Chars,
    check_ending: F,
    include_ending: bool,
) -> Result<String, ParseHtmlError> {
    let mut buffer = String::with_capacity(50);
    while let Some(ch) = chs.next() {
        if check_ending(&ch) {
            if include_ending {
                buffer.push(ch);
            }
            return Ok(buffer);
        }
        buffer.push(ch);
    }
    return Err(ParseHtmlError::new(format!(
        "End of string '{}' encountered before ending was found",
        buffer
    )));
}

fn parse_attibute_value(attr_value: String) -> Vec<String> {
    let mut v = vec![];
    for val in attr_value.split_ascii_whitespace() {
        v.push(val.to_string());
    }
    return v;
}

fn get_next_non_whitespace(chs: &mut std::str::Chars) -> Result<char, ParseHtmlError> {
    while let Some(ch) = chs.next() {
        if !ch.is_ascii_whitespace() {
            return Ok(ch);
        }
    }
    return Err(ParseHtmlError::new(format!(
        "End found while consuming whitespace."
    )));
}

#[cfg(test)]
mod parse_iter_tests {
    use super::*;

    #[test]
    fn parse_chars_test() {
        assert_eq!(
            parse_until_char(&mut "Something else".chars(), ' ', false).unwrap(),
            "Something"
        );
        assert_eq!(
            parse_until_char(&mut "Something else".chars(), ' ', true).unwrap(),
            "Something "
        );
        assert_eq!(
            parse_until_one_of(&mut "Something else <".chars(), vec![' ', '<'], false).unwrap(),
            "Something"
        );
        assert_eq!(
            parse_until_one_of(&mut "Something else <".chars(), vec![' ', '<'], true).unwrap(),
            "Something "
        );
        assert_eq!(
            parse_until_one_of(&mut "Something else <".chars(), vec!['<'], false).unwrap(),
            "Something else "
        );
        assert_eq!(
            parse_until_one_of(&mut "Something else <".chars(), vec!['<'], true).unwrap(),
            "Something else <"
        );
        assert_eq!(
            parse_until_str(&mut "Something else <".chars(), &"else", false).unwrap(),
            "Something "
        );
        assert_eq!(
            parse_until_str(&mut "Something else <".chars(), &"else", true).unwrap(),
            "Something else"
        );
        let cl = |c: &char| -> bool {
            return c.is_ascii_whitespace();
        };
        assert_eq!(
            parse_until(&mut "Something else".chars(), cl, false).unwrap(),
            "Something"
        );
        assert_eq!(
            parse_until(&mut "Something else".chars(), cl, true).unwrap(),
            "Something "
        );

        assert_eq!(
            parse_string(&mut " test ".chars()).unwrap(),
            ("test".to_string(), ' ')
        );
        assert_eq!(
            parse_string(&mut " test>".chars()).unwrap(),
            ("test".to_string(), '>')
        );
        assert_eq!(
            parse_string(&mut " \"test \"".chars()).unwrap(),
            ("test ".to_string(), '"')
        );
        assert_eq!(
            parse_string(&mut "\"test \"".chars()).unwrap(),
            ("test ".to_string(), '"')
        );
    }
}

#[derive(Debug, PartialEq)]
enum ParsedTagType {
    EndTag(String),  //eg </div>
    NewTag(HtmlTag), //eg <div class="test">
    Comment(String), //eg <!-- text --!>
    DocType(String),
}

fn parse_html_tag(chs: &mut std::str::Chars) -> Result<ParsedTagType, ParseHtmlError> {
    // read chars into the buffer until a > or ' ' is found
    let mut buffer = String::with_capacity(50);
    //read first character and determine if this is an end tag
    match chs.next() {
        Some(ch) => {
            if ch == '/' {
                return Ok(ParsedTagType::EndTag(
                    parse_until_char(chs, '>', false)?.trim_end().to_owned(),
                ));
            } else {
                buffer.push(ch);
            }
        }
        None => {
            return Err(ParseHtmlError::new(format!(
                "End of file without reading any chars in this tag."
            )));
        }
    }
    //Parse until we get the end of tag or a space
    buffer.push_str(parse_until_one_of(chs, vec![' ', '>'], true)?.as_str());
    if buffer.starts_with("!--") {
        // comment - parse the rest of the comment
        buffer.drain(0..3);
        if buffer.ends_with("-->") {
            buffer.truncate(buffer.len() - 4);
            return Ok(ParsedTagType::Comment(buffer));
        }
        buffer.push_str(parse_until_str(chs, &"-->", false)?.as_str());
        return Ok(ParsedTagType::Comment(buffer));
    } else if buffer.starts_with("!DOCTYPE ") {
        buffer.clear();
        return Ok(ParsedTagType::DocType(parse_until_char(chs, '>', false)?));
    }
    let (tag_str, ending) = buffer.split_at(buffer.len() - 1);
    let tag = tag_str.to_owned();
    let mut node = HtmlTag::new(tag_str);
    if ending != ">" {
        //define the some checking closures
        let is_ws_eq_or_gt = |ch: &char| -> bool {
            return ch.is_ascii_whitespace() || *ch == '=' || *ch == '>';
        };
        loop {
            buffer.clear();
            buffer.push(get_next_non_whitespace(chs)?);
            if buffer == ">" {
                //didn't get an attribute - just got the end of tag symbol
                break;
            }
            buffer.push_str(parse_until(chs, is_ws_eq_or_gt, true)?.as_str());
            let (attr_str, attr_ending) = buffer.split_at(buffer.len() - 1);
            if attr_ending == ">" {
                if attr_str.len() > 0 {
                    return Err(ParseHtmlError::new(format!(
                        "Expected value for attribute '{}', got {} instead of '='.",
                        attr_str, attr_ending
                    )));
                }
                //didn't get an attribute - just got the end of tag symbol
                break;
            }
            if attr_ending.chars().next().unwrap().is_ascii_whitespace() {
                //clear all whitespace until we get a '='
                let ch = get_next_non_whitespace(chs)?;
                if ch != '=' {
                    return Err(ParseHtmlError::new(format!(
                        "Expected value for attribute '{}', got {} instead of '='.",
                        attr_str, ch
                    )));
                }
            }
            //We have 'attr =' now need to read in the value
            let (attr_value_string, attr_value_ending) = parse_string(chs)?;
            if attr_str == "class" {
                node.classes = parse_attibute_value(attr_value_string);
            } else if attr_str == "id" {
                node.ids = parse_attibute_value(attr_value_string);
            } else {
                node.attributes
                    .insert(attr_str.to_string(), attr_value_string);
            }
            if attr_value_ending == '>' {
                break;
            }
        }
    }
    //Return the node without content if it is a singleton tag
    match tag.as_str() {
        "area" => return Ok(ParsedTagType::NewTag(node)),
        "base" => return Ok(ParsedTagType::NewTag(node)),
        "br" => return Ok(ParsedTagType::NewTag(node)),
        "col" => return Ok(ParsedTagType::NewTag(node)),
        "command" => return Ok(ParsedTagType::NewTag(node)),
        "embed" => return Ok(ParsedTagType::NewTag(node)),
        "hr" => return Ok(ParsedTagType::NewTag(node)),
        "img" => return Ok(ParsedTagType::NewTag(node)),
        "input" => return Ok(ParsedTagType::NewTag(node)),
        "keygen" => return Ok(ParsedTagType::NewTag(node)),
        "link" => return Ok(ParsedTagType::NewTag(node)),
        "meta" => return Ok(ParsedTagType::NewTag(node)),
        "param" => return Ok(ParsedTagType::NewTag(node)),
        "source" => return Ok(ParsedTagType::NewTag(node)),
        "track" => return Ok(ParsedTagType::NewTag(node)),
        "wbr" => return Ok(ParsedTagType::NewTag(node)),
        _ => (),
    }
    node.contents = parse_html_content(chs, tag)?;
    Ok(ParsedTagType::NewTag(node))
}
#[cfg(test)]
mod parse_html_tag_tests {
    use super::*;

    //TODO: How to test failing cases...
    #[test]
    fn parse_html_start_tag_test() {
        assert_eq!(
            parse_html_tag(&mut "div></div>".chars()).unwrap(),
            ParsedTagType::NewTag(HtmlTag::new("div"))
        );
        let mut tag = HtmlTag::new("a");
        tag.classes.push("class1".to_string());
        tag.ids.push("id1".to_string());
        tag.contents
            .push(HtmlNode::Text("Some Content".to_string()));
        tag.attributes
            .insert("other_attr".to_string(), "something".to_string());
        assert_eq!(
            parse_html_tag(
                &mut "a class=\"class1\" id = \"id1\" other_attr=something>Some Content</a>"
                    .chars()
            )
            .unwrap(),
            ParsedTagType::NewTag(tag)
        );
    }

    #[test]
    fn parse_html_end_tag_test() {
        assert_eq!(
            parse_html_tag(&mut "/div>".chars()).unwrap(),
            ParsedTagType::EndTag("div".to_string())
        );
        assert_eq!(
            parse_html_tag(&mut "/div >".chars()).unwrap(),
            ParsedTagType::EndTag("div".to_string())
        );
    }

    #[test]
    fn parse_html_comment_tag_test() {
        assert_eq!(
            parse_html_tag(&mut "!-- something -->".chars()).unwrap(),
            ParsedTagType::Comment(" something ".to_string())
        );
        assert_eq!(
            parse_html_tag(&mut "!-- something\n something else -->".chars()).unwrap(),
            ParsedTagType::Comment(" something\n something else ".to_string())
        );
    }
}

pub fn parse_html_content(
    chs: &mut std::str::Chars,
    tag: String,
) -> Result<Vec<HtmlNode>, ParseHtmlError> {
    let mut text_content = String::new();
    let mut content: Vec<HtmlNode> = Vec::new();
    while let Some(cur_char) = chs.next() {
        if cur_char == '<' {
            if text_content.len() > 0 {
                content.push(HtmlNode::Text(text_content));
                text_content = String::new();
            }
            //Read rest of tag - passing along any errors that were encountered.
            match parse_html_tag(chs)? {
                ParsedTagType::EndTag(end_tag) => {
                    if end_tag != tag {
                        return Err(ParseHtmlError::new(format!(
                            "Incorrect end tag found {} but expected {}.",
                            end_tag, tag
                        )));
                    }
                    //Got the correct end tag
                    if content.len() > 0 {
                        return Ok(content);
                    } else {
                        return Ok(vec![]);
                    }
                }
                ParsedTagType::NewTag(node_rc) => {
                    content.push(HtmlNode::Tag(node_rc));
                }
                ParsedTagType::Comment(comment) => {
                    content.push(HtmlNode::Comment(comment));
                }
                ParsedTagType::DocType(t) => {
                    return Err(ParseHtmlError::new(format!(
                        "'DOCTYPE {}' element found in middle of content",
                        t
                    )))
                }
            }
        } else {
            text_content.push(cur_char);
        }
    }
    //Parse HTML until end tag </tag> is found
    return Err(ParseHtmlError::new(format!(
        "End of file without finding tag {}.",
        tag
    )));
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

#[cfg(test)]
mod parse_html_document_tests {
    use super::*;

    #[test]
    fn parse_test_document() {
        let test_html = r#"<!DOCTYPE html>
<!-- saved from url=(0117)https://www.webfx.com/blog/images/assets/cdn.sixrevisions.com/0435-01_html5_download_attribute_demo/samp/htmldoc.html -->
<html><head><meta http-equiv="Content-Type" content="text/html; charset=UTF-8">
<title>A Sample HTML Document (Test File)</title>
<meta name="description" content="A blank HTML document for testing purposes.">
<meta name="author" content="Six Revisions">
<meta name="viewport" content="width=device-width, initial-scale=1">
<link rel="icon" href="http://sixrevisions.com/favicon.ico" type="image/x-icon">
</head>
<body>
    
<h1 class=heading>A Sample HTML Document (Test File)</h1>
<p>A blank HTML document for testing purposes.</p>
<p><a href="https://www.webfx.com/blog/images/assets/cdn.sixrevisions.com/0435-01_html5_download_attribute_demo/html5download-demo.html">Go back to the demo</a></p>
<p><a href="http://sixrevisions.com/html5/download-attribute/">Read the HTML5 download attribute guide</a></p>
</body></html>"#;
        let doc_from_str = HtmlDocument::from_str(test_html).unwrap();
        assert_eq!(doc_from_str.doctype, "html".to_owned());
        let mut doc = HtmlDocument::new();
        doc.nodes.push(HtmlNode::Comment(" saved from url=(0117)https://www.webfx.com/blog/images/assets/cdn.sixrevisions.com/0435-01_html5_download_attribute_demo/samp/htmldoc.html ".to_string()));

        let mut html_tag = HtmlTag::new("html");
        let mut head = HtmlTag::new("head");
        let mut meta = HtmlTag::new("meta");
        meta.attributes
            .insert("http-equiv".to_owned(), "Content-Type".to_string());
        meta.attributes
            .insert("content".to_owned(), "text/html; charset=UTF-8".to_string());
        head.contents.push(HtmlNode::Tag(meta));
        head.contents.push(HtmlNode::Text("\n".to_string()));

        let mut title = HtmlTag::new("title");
        title.contents.push(HtmlNode::Text(
            "A Sample HTML Document (Test File)".to_string(),
        ));
        head.contents.push(HtmlNode::Tag(title));
        head.contents.push(HtmlNode::Text("\n".to_string()));

        let mut meta = HtmlTag::new("meta");
        meta.attributes
            .insert("name".to_string(), "description".to_string());
        meta.attributes.insert(
            "content".to_string(),
            "A black HTML document for testing purposes.".to_string(),
        );
        head.contents.push(HtmlNode::Tag(meta));
        head.contents.push(HtmlNode::Text("\n".to_string()));
        let mut meta = HtmlTag::new("meta");
        meta.attributes
            .insert("name".to_owned(), "author".to_string());
        meta.attributes
            .insert("content".to_owned(), "Six Revisions".to_string());
        head.contents.push(HtmlNode::Tag(meta));
        head.contents.push(HtmlNode::Text("\n".to_string()));
        let mut meta = HtmlTag::new("meta");
        meta.attributes
            .insert("name".to_owned(), "viewport".to_string());
        meta.attributes.insert(
            "content".to_owned(),
            "width=device-width, initial-scale=1".to_string(),
        );
        head.contents.push(HtmlNode::Tag(meta));
        head.contents.push(HtmlNode::Text("\n".to_string()));
        let mut link = HtmlTag::new("link");
        link.attributes
            .insert("rel".to_string(), "icon".to_string());
        link.attributes.insert(
            "href".to_string(),
            "http://sixrevisions.com/favicon.ico".to_string(),
        );
        link.attributes
            .insert("type".to_string(), "image/x-icon".to_string());
        head.contents.push(HtmlNode::Tag(link));
        head.contents.push(HtmlNode::Text("\n".to_string()));

        html_tag.contents.push(HtmlNode::Tag(head));
        html_tag.contents.push(HtmlNode::Text("\n".to_string()));

        let mut body = HtmlTag::new("body");
        body.contents.push(HtmlNode::Text("\n    \n".to_string()));
        let mut h1 = HtmlTag::new("h1");
        h1.classes.push("heading".to_string());
        h1.contents.push(HtmlNode::Text(
            "A Sample HTML Document (Test File)".to_string(),
        ));
        body.contents.push(HtmlNode::Tag(h1));
        body.contents.push(HtmlNode::Text("\n".to_string()));
        let mut p = HtmlTag::new("p");
        p.contents.push(HtmlNode::Text(
            "A black HTML document for testing purposes.".to_string(),
        ));
        body.contents.push(HtmlNode::Tag(p));
        body.contents.push(HtmlNode::Text("\n".to_string()));
        let mut p = HtmlTag::new("p");
        let mut a = HtmlTag::new("a");
        a.attributes.insert("href".to_string(), "https://www.webfx.com/blog/images/assets/cdn.sixrevisions.com/0435-01_html5_download_attribute_demo/html5download-demo.html".to_string());
        a.contents
            .push(HtmlNode::Text("Go back to the demo".to_string()));
        p.contents.push(HtmlNode::Tag(a));
        body.contents.push(HtmlNode::Tag(p));
        body.contents.push(HtmlNode::Text("\n".to_string()));
        let mut p = HtmlTag::new("p");
        let mut a = HtmlTag::new("a");
        a.attributes.insert(
            "href".to_string(),
            "http://sixrevisions.com/html5/download-attribute/".to_string(),
        );
        a.contents.push(HtmlNode::Text(
            "Read the HTML5 download attribute guide".to_string(),
        ));
        p.contents.push(HtmlNode::Tag(a));
        body.contents.push(HtmlNode::Tag(p));
        body.contents.push(HtmlNode::Text("\n".to_string()));
        html_tag.contents.push(HtmlNode::Tag(body));
        doc.nodes.push(HtmlNode::Tag(html_tag));

        assert_eq!(doc.nodes, doc_from_str.nodes);
    }
}

/// An object which points to the a node in the HTML tree including the path to
/// the node to allow looking at parent nodes.
struct HtmlQueryResult<'a> {
    /// The path down the tree.
    /// The node is found by dereferencing the last element of the vector
    path: Vec<(&'a Vec<HtmlNode>, usize)>,
}

impl<'a> HtmlQueryResult<'a> {
    /// Attempts to get the node pointed to by the path.
    /// Returns None if the path is empty.
    fn get_node(&self) -> Option<&HtmlNode> {
        if self.path.len() == 0 {
            return None;
        }
        let path_point = &self.path[self.path.len() - 1];
        return Some(&path_point.0[path_point.1]);
    }

    /// Attempts to get the parent to the node pointed to by the path.
    /// Returns None if the path is empty or is only the single node on the path.
    fn get_parent_node(&self) -> Option<&HtmlNode> {
        if self.path.len() <= 1 {
            return None;
        }
        let path_point = &self.path[self.path.len() - 2];
        return Some(&path_point.0[path_point.1]);
    }

    /// Attempts to get the node from the index position on the path.
    /// Returns None if the index is out of range of the path.
    fn get_node_from_index(&self, index: usize) -> Option<&HtmlNode> {
        if index >= self.path.len() {
            return None;
        }
        let path_point = &self.path[index];
        return Some(&path_point.0[path_point.1]);
    }

    /// Creates an iterator that walks the path from the bottom to the top.
    fn get_path_iter(&self) -> HtmlQueryResultIter {
        HtmlQueryResultIter::new(self)
    }
}

/// Iterator that walks along the path of the HtmlQueryResult from the bottom to
/// the top.
struct HtmlQueryResultIter<'a> {
    query_result: &'a HtmlQueryResult<'a>,
    previous_index: usize,
}
impl<'a> HtmlQueryResultIter<'a> {
    /// Create a HtmlQueryResultIter from a HtmlQueryResult reference.
    fn new(query_result: &'a HtmlQueryResult) -> HtmlQueryResultIter<'a> {
        HtmlQueryResultIter {
            query_result: query_result,
            previous_index: query_result.path.len(),
        }
    }
}
impl<'a> Iterator for HtmlQueryResultIter<'a> {
    type Item = &'a HtmlNode;
    fn next(&mut self) -> Option<Self::Item> {
        if self.previous_index == 0 {
            return None;
        }
        self.previous_index = self.previous_index - 1;
        self.query_result.get_node_from_index(self.previous_index)
    }
}

/// Allows searching through HTML documents using various search functions.
/// Results are stores as HtmlQueryResults.
///
/// Multiple searches are allowed on a single HtmlQuery object, and each
/// subsequent search will search from the existing results.
/// For example:
/// ```
/// let query = HtmlQuery::new(&html_doc.nodes);
/// query.find_with_tag("div").find_with_tag("p")
/// ```
/// This will find the div tags, then find the p tags from within the div tags.
struct HtmlQuery<'a> {
    root: &'a Vec<HtmlNode>,
    results: Vec<HtmlQueryResult<'a>>,
}

impl<'a> HtmlQuery<'a> {
    /// Creates a new HtmlQuery to search from the root nodes down.
    ///
    /// # Arguments
    ///
    /// * `root` - A reference to the vector of nodes that the Query object
    ///            begins searching from.
    fn new(root: &'a Vec<HtmlNode>) -> HtmlQuery<'a> {
        HtmlQuery {
            root: root,
            results: vec![],
        }
    }

    ///
    fn find_with_tag(&self, tag: &str) -> &HtmlQuery {
        todo!();
    }
}
struct HtmlQueryResultMut<'a> {
    path: Vec<(&'a mut Vec<HtmlNode>, usize)>,
}

struct HtmlQueryMut<'a> {
    root: &'a mut Vec<HtmlNode>,
    results: Vec<HtmlQueryResultMut<'a>>,
}

impl<'a> HtmlQueryMut<'a> {
    fn new(root: &'a mut Vec<HtmlNode>) -> HtmlQueryMut<'a> {
        HtmlQueryMut {
            root: root,
            results: vec![],
        }
    }
}
