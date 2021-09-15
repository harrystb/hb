use std::cell::RefCell;
use std::collections::VecDeque;
use std::collections::HashMap;
use std::str::FromStr;
use std::rc::{Weak, Rc};

pub struct ParseHtmlError {
    msg : String,
}

impl ParseHtmlError {
    fn new(msg : String) -> ParseHtmlError {
        ParseHtmlError { msg : msg}
    }

    fn with_msg<S : Into<String>>(msg : S) -> ParseHtmlError {
        return ParseHtmlError::new(msg.into());
    }
}

impl std::fmt::Display for ParseHtmlError {
    fn fmt(&self, f : &mut  std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        write!(f, "Error occured parsing HTML '{}'", self.msg)?;
        Ok(())
    }
}
impl std::fmt::Debug for ParseHtmlError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        write!(f, "Error occured parsing HTML '{}'", self.msg)?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct HtmlDocument {
    doctype : Option<String>,
    nodes : Option<Vec<HtmlContent>>,
}

#[derive(Debug, Clone)]
pub struct HtmlNode {
    tag : String,
    contents : Option<Vec<HtmlContent>>,
    class : Option<Vec<String>>,
    id : Option<Vec<String>>,
    attributes : Option<HashMap<String, String>>,
    parent : Option<Weak<RefCell<HtmlNode>>>,
}

impl PartialEq for HtmlNode {

    fn eq(&self, other: &Self) -> bool 
    {
        self.id == other.id &&
        self.attributes == other.attributes &&
        self.class == other.class &&
        self.contents == other.contents &&
        self.tag == other.tag &&
        (match &self.parent {
            Some(parent) => {
                match &other.parent {
                    Some(other_parent) => parent.ptr_eq(&other_parent),
                    None => false,
                }},
            None => {
                match other.parent {
                    Some(_) => false,
                    None => true,
                }
            }
        })  
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum HtmlContent {
    Text(String),
    Node(Rc<RefCell<HtmlNode>>),
    Comment(String),
}

#[derive(PartialEq, Debug)]
struct CSSSelectorData {
    class : Option<String>,
    id : Option<String>,
    tag : Option<String>,
}
#[derive(PartialEq, Debug)]
enum CSSSelector {
    All,
    Specific(Vec<CSSSelectorData>),
}

impl CSSSelector {
    fn new<S: Into<String>>(s : S) -> Result<Self, ParseHtmlError>{
        let s = s.into();
        if s == "*" {
            return Ok(CSSSelector::All);
        }
        let mut output = vec![];
        for word in s.split_ascii_whitespace() {
            if word.starts_with('#') {
                output.push(CSSSelectorData {class: None, id: Some(word[1..].to_string()), tag : None});
            } else {
                match word.split_once('.') {
                    //TODO: need to some checks for empty class, etc...
                    Some((tag, class)) => {
                        output.push(CSSSelectorData { class : Some(class.to_string()), id : None, tag : if tag == "" { None } else { Some(tag.to_string()) } });
                    },
                    None => {
                        //tag only
                        output.push(CSSSelectorData {class: None, id: None, tag : Some(word.to_string())});
                    },
                }
            }
        }
        if output.len() > 0 {
            return Ok(CSSSelector::Specific(output));
        }
        return Err(ParseHtmlError::new(format!("Unknown selector {}", s)));    

    }
}

impl FromStr for CSSSelector {
    type Err = ParseHtmlError;
    fn from_str(s: &str) -> std::result::Result<Self, <Self as std::str::FromStr>::Err> {
        CSSSelector::new(s)
    }
}

#[cfg(test)]
mod css_selector_tests {
    use super::*;

    #[test]
    fn create_selector_from_string_test() -> Result<(), ParseHtmlError> {
        let selector = CSSSelector::from_str(".test")?;
        assert_eq!(selector, CSSSelector::Specific(vec![CSSSelectorData{class : Some("test".to_string()), id : None, tag : None }]));
        let selector = CSSSelector::from_str("tg.test")?;
        assert_eq!(selector, CSSSelector::Specific(vec![CSSSelectorData{class : Some("test".to_string()), id : None, tag : Some("tg".to_string()) }]));
        let selector = CSSSelector::from_str("tg.test .cl")?;
        assert_eq!(selector, CSSSelector::Specific(vec![CSSSelectorData{class : Some("test".to_string()), id : None, tag : Some("tg".to_string()) }, CSSSelectorData{class : Some("cl".to_string()), id : None, tag : None }]));
        let selector = CSSSelector::from_str("#isee")?;
        assert_eq!(selector, CSSSelector::Specific(vec![CSSSelectorData{class : None, id : Some("isee".to_string()), tag : None }]));
        Ok(())
    }

}

impl HtmlNode {
    fn new<S: Into<String>>(tag : S) -> HtmlNode {
        HtmlNode { tag : tag.into(), class : None, id : None, contents : None, attributes : None, parent : None}
    }

}

#[cfg(test)]
mod html_node_tests {
    use super::*;

    #[test]
    fn node_selection() {
        let mut root = HtmlNode::new("html");
        let mut node1 = HtmlNode::new("div");
        node1.class = Some(vec!["test".to_string(), "test-2".to_string()]);
        let mut node2 = HtmlNode::new("div");
        node2.class = Some(vec!["test".to_string()]);

        root.contents = Some(vec![HtmlContent::Node(Rc::new(RefCell::new(node1))), HtmlContent::Text("some_text.omg".to_string()), HtmlContent::Node(Rc::new(RefCell::new(node2)))]);
        //complete
    }
}


// Read from the iterator until a quoted string or word is found (ignoring leading whitespace) then return the string and the character that ended the string
// Endings of a single word can be whitespace or >
fn parse_string(chs : &mut std::str::Chars) -> Result<(String, char), ParseHtmlError> {
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
    return Err(ParseHtmlError::new(format!("End of string '{}' not found", buffer)));
}

fn parse_until_one_of(chs : &mut std::str::Chars, end_chars : Vec<char>, include_ending : bool) -> Result<String, ParseHtmlError> {
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
    return Err(ParseHtmlError::new(format!("End of string '{}' encountered before any end char '{:?}' was found", buffer, end_chars)));
}

fn parse_until_char(chs : &mut std::str::Chars, end_char : char, include_ending : bool) -> Result<String, ParseHtmlError> {
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
    return Err(ParseHtmlError::new(format!("End of string '{}' encountered before end char '{}' was found", buffer, end_char)));
}

fn parse_until_str(chs : &mut std::str::Chars, end_str: &str, include_ending : bool) -> Result<String, ParseHtmlError> {
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
    return Err(ParseHtmlError::new(format!("End of string '{}{}' encountered before any end string '{}' was found", buffer, ending_buffer.iter().collect::<String>(), end_str)));           
}

fn parse_until<F : Fn(&char) -> bool>(chs : &mut std::str::Chars, check_ending : F, include_ending : bool) -> Result<String, ParseHtmlError> {
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
    return Err(ParseHtmlError::new(format!("End of string '{}' encountered before ending was found", buffer)));
}

fn parse_attibute_value (attr_value : String) -> Vec<String> {
    let mut v = vec![];
    for val in attr_value.split_ascii_whitespace() {
        v.push(val.to_string());
    }
    return v;
}

fn get_next_non_whitespace(chs : &mut std::str::Chars) -> Result<char, ParseHtmlError> {
    while let Some(ch) = chs.next() {
        if !ch.is_ascii_whitespace() {
            return Ok(ch);
        }
    }
    return Err(ParseHtmlError::new(format!("End found while consuming whitespace.")));
}

#[cfg(test)]
mod parse_iter_tests {
    use super::*;

    #[test]
    fn parse_chars_test() {
        assert_eq!(parse_until_char(&mut "Something else".chars(), ' ', false).unwrap(), "Something");
        assert_eq!(parse_until_char(&mut "Something else".chars(), ' ', true).unwrap(), "Something ");
        assert_eq!(parse_until_one_of(&mut "Something else <".chars(), vec![' ', '<'], false).unwrap(), "Something");
        assert_eq!(parse_until_one_of(&mut "Something else <".chars(), vec![' ', '<'], true).unwrap(), "Something ");
        assert_eq!(parse_until_one_of(&mut "Something else <".chars(), vec!['<'], false).unwrap(), "Something else ");
        assert_eq!(parse_until_one_of(&mut "Something else <".chars(), vec!['<'], true).unwrap(), "Something else <");
        assert_eq!(parse_until_str(&mut "Something else <".chars(), &"else", false).unwrap(), "Something ");
        assert_eq!(parse_until_str(&mut "Something else <".chars(), &"else", true).unwrap(), "Something else");
        let cl = |c : &char| -> bool {
            return c.is_ascii_whitespace();
        };
        assert_eq!(parse_until(&mut "Something else".chars(), cl, false).unwrap(), "Something");
        assert_eq!(parse_until(&mut "Something else".chars(), cl, true).unwrap(), "Something ");

        assert_eq!(parse_string(&mut " test ".chars()).unwrap(), ("test".to_string(), ' '));
        assert_eq!(parse_string(&mut " test>".chars()).unwrap(), ("test".to_string(), '>'));
        assert_eq!(parse_string(&mut " \"test \"".chars()).unwrap(), ("test ".to_string(), '"'));
        assert_eq!(parse_string(&mut "\"test \"".chars()).unwrap(), ("test ".to_string(), '"'));
    }
}

#[derive(Debug, PartialEq)]
enum HtmlTag {
    EndTag(String), //eg </div>
    NewTag(Rc<RefCell<HtmlNode>>), //eg <div class="test">
    Comment(String), //eg <!-- text --!>
    DocType(String),
}

fn parse_html_tag(chs : &mut std::str::Chars, parent_weak_rc : Option<Weak<RefCell<HtmlNode>>>) -> Result<HtmlTag, ParseHtmlError> {
    // read chars into the buffer until a > or ' ' is found
    let mut buffer = String::with_capacity(50);
    //read first character and determine if this is an end tag 
    match chs.next() {
        Some(ch) => {
            if ch == '/' {
                return Ok(HtmlTag::EndTag(parse_until_char(chs, '>', false)?.trim_end().to_owned()));
            } else {
                buffer.push(ch);
            }
        },
        None => {
            return Err(ParseHtmlError::new(format!("End of file without reading any chars in this tag.")));
        }
    }
    //Parse until we get the end of tag or a space
    buffer.push_str(parse_until_one_of(chs, vec![' ','>'], true)?.as_str());
    if buffer.starts_with("!--") {
        // comment - parse the rest of the comment
        buffer.drain(0..3);
        if buffer.ends_with("-->") {
            buffer.truncate(buffer.len()-4);
            return Ok(HtmlTag::Comment(buffer));
        }
        buffer.push_str(parse_until_str(chs, &"-->", false)?.as_str());
        return Ok(HtmlTag::Comment(buffer))
    } else if buffer.starts_with("!DOCTYPE ") {
        buffer.clear();
        return Ok(HtmlTag::DocType(parse_until_char(chs, '>', false)?));
    }
    let (tag_str, ending) = buffer.split_at(buffer.len()-1);
    let mut node_rc = Rc::new(RefCell::new(HtmlNode::new(tag_str)));
    match parent_weak_rc {
        Some(weak_rc) => node_rc.borrow_mut().parent = Some(weak_rc.clone()),
        None => (),
    }
    if ending != ">" {
        let mut node = node_rc.borrow_mut();
        //define the some checking closures
        let is_ws_eq_or_gt = |ch : &char| -> bool {
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
            let (attr_str, attr_ending) = buffer.split_at(buffer.len()-1);
            if attr_ending == ">" {
                if attr_str.len() > 0 {
                    return Err(ParseHtmlError::new(format!("Expected value for attribute '{}', got {} instead of '='.", attr_str, attr_ending)));
                }
                //didn't get an attribute - just got the end of tag symbol
                break;
            }
            if attr_ending.chars().next().unwrap().is_ascii_whitespace()  {
                //clear all whitespace until we get a '='
                let ch = get_next_non_whitespace(chs)?;
                if ch != '=' {
                    return Err(ParseHtmlError::new(format!("Expected value for attribute '{}', got {} instead of '='.", attr_str, ch)));
                }
            }
            //We have 'attr =' now need to read in the value
            let (attr_value_string, attr_value_ending) = parse_string(chs)?;
            if attr_str == "class" {
                node.class = Some(parse_attibute_value(attr_value_string));
            } else if attr_str == "id" {
                node.id = Some(parse_attibute_value(attr_value_string));
            } else {
                match &mut node.attributes {
                    Some(attr_map) => {
                        attr_map.insert(attr_str.to_string(), attr_value_string);
                    },
                    None => {
                        let mut attr_map = HashMap::new();
                        attr_map.insert(attr_str.to_string(), attr_value_string);
                        node.attributes = Some(attr_map);
                    }
                }
            }
            if attr_value_ending == '>' {
                break;
            }
        }
    }
    //Return the node without content if it is a singleton tag
    let tag = node_rc.borrow().tag.to_owned();
    match tag.as_str() {
        "area" => return Ok(HtmlTag::NewTag(node_rc)),
        "base" => return Ok(HtmlTag::NewTag(node_rc)),
        "br" => return Ok(HtmlTag::NewTag(node_rc)),
        "col" => return Ok(HtmlTag::NewTag(node_rc)),
        "command" => return Ok(HtmlTag::NewTag(node_rc)),
        "embed" => return Ok(HtmlTag::NewTag(node_rc)),
        "hr" => return Ok(HtmlTag::NewTag(node_rc)),
        "img" => return Ok(HtmlTag::NewTag(node_rc)),
        "input" => return Ok(HtmlTag::NewTag(node_rc)),
        "keygen" => return Ok(HtmlTag::NewTag(node_rc)),
        "link" => return Ok(HtmlTag::NewTag(node_rc)),
        "meta" => return Ok(HtmlTag::NewTag(node_rc)),
        "param" => return Ok(HtmlTag::NewTag(node_rc)),
        "source" => return Ok(HtmlTag::NewTag(node_rc)),
        "track" => return Ok(HtmlTag::NewTag(node_rc)),
        "wbr" => return Ok(HtmlTag::NewTag(node_rc)),
        _ => {
            node_rc.borrow_mut().contents = parse_html_content(chs, tag.as_str(), Some(Rc::downgrade(&node_rc)))?;
            Ok(HtmlTag::NewTag(node_rc))
        }
    }
}
#[cfg(test)]
mod parse_html_tag_tests {
    use super::*;

    //TODO: How to test failing cases...
    #[test]
    fn parse_html_start_tag_test() {
        assert_eq!(parse_html_tag(&mut "div></div>".chars(), None).unwrap(), HtmlTag::NewTag(Rc::new(RefCell::new(HtmlNode { tag: "div".to_string(), class : None, id : None, attributes : None, contents : None, parent : None }))));
        let node_rc = Rc::new(RefCell::new(HtmlNode::new("a")));
        assert_eq!(parse_html_tag(&mut "div></div>".chars(), Some(Rc::downgrade(&node_rc))).unwrap(), HtmlTag::NewTag(Rc::new(RefCell::new(HtmlNode { tag: "div".to_string(), class : None, id : None, attributes : None, contents : None, parent : Some(Rc::downgrade(&node_rc)) }))));
        assert_eq!(
            parse_html_tag(&mut "div  class=\"class1\">Some Content</div>".chars(), None).unwrap(), 
            HtmlTag::NewTag(Rc::new(RefCell::new(HtmlNode { tag: "div".to_string(), class : Some(vec!["class1".to_string()]), id : None, attributes : None, contents : Some(vec![HtmlContent::Text("Some Content".to_owned())]), parent : None})))
        );
        assert_eq!(
            parse_html_tag(&mut "div  class=\"class1\" ></div>".chars(), None).unwrap(), 
            HtmlTag::NewTag(Rc::new(RefCell::new(HtmlNode { tag: "div".to_string(), class : Some(vec!["class1".to_string()]), id : None, attributes : None, contents : None, parent : None })))
        );
        assert_eq!(
            parse_html_tag(&mut "a class=\"class1\" id = \"id1\"></a>".chars(), None).unwrap(), 
            HtmlTag::NewTag(Rc::new(RefCell::new(HtmlNode { tag: "a".to_string(), class : Some(vec!["class1".to_string()]), id : Some(vec!["id1".to_string()]), attributes : None, contents : None, parent : None })))
        );
        assert_eq!(
            parse_html_tag(&mut "a class=\"class1\" id = \"id1\" ></a>".chars(), None).unwrap(), 
            HtmlTag::NewTag(Rc::new(RefCell::new(HtmlNode { tag: "a".to_string(), class : Some(vec!["class1".to_string()]), id : Some(vec!["id1".to_string()]), attributes : None, contents : None , parent : None})))
        );
        let mut map = HashMap::new();
        map.insert("other_attr".to_string(), "something".to_string());
        assert_eq!(
            parse_html_tag(&mut "a class=\"class1\" other_attr=something></a>".chars(), None).unwrap(), 
            HtmlTag::NewTag(Rc::new(RefCell::new(HtmlNode { tag: "a".to_string(), class : Some(vec!["class1".to_string()]), id : None, attributes : Some(map), contents : None , parent : None})))
        );

    }

    #[test]
    fn parse_html_end_tag_test() {
        assert_eq!(parse_html_tag(&mut "/div>".chars(), None).unwrap(), HtmlTag::EndTag("div".to_string()));
        assert_eq!(parse_html_tag(&mut "/div >".chars(), None).unwrap(), HtmlTag::EndTag("div".to_string()));
    }

    #[test]
    fn parse_html_comment_tag_test() {
        assert_eq!(
            parse_html_tag(&mut "!-- something -->".chars(), None).unwrap(), 
            HtmlTag::Comment(" something ".to_string())
        );
        assert_eq!(
            parse_html_tag(&mut "!-- something\n something else -->".chars(), None).unwrap(), 
            HtmlTag::Comment(" something\n something else ".to_string())
        );
    }
}

pub fn parse_html_content(chs : &mut std::str::Chars, tag : &str, parent_weak_rc : Option<Weak<RefCell<HtmlNode>>>) -> Result<Option<Vec<HtmlContent>>, ParseHtmlError> {
    let mut text_content = String::new();
    let mut content : Vec<HtmlContent> = Vec::new();
    while let Some(cur_char) = chs.next() {
        if cur_char == '<' {
            if text_content.len() > 0 {
                content.push(HtmlContent::Text(text_content));
                text_content = String::new();
            }
            //Read rest of tag - passing along any errors that were encountered.
            match parse_html_tag(chs, parent_weak_rc.clone())? {
                HtmlTag::EndTag(end_tag) => {
                    if end_tag != tag {
                        return Err(ParseHtmlError::new(format!("Incorrect end tag found {} but expected {}.", end_tag, tag)));
                    }
                    //Got the correct end tag
                    if content.len() > 0 {
                        return Ok(Some(content));
                    } else {
                        return Ok(None);
                    }
                },
                HtmlTag::NewTag(node_rc) => {
                    content.push(HtmlContent::Node(node_rc));
                },
                HtmlTag::Comment(comment) => {
                    content.push(HtmlContent::Comment(comment));
                },
                HtmlTag::DocType(t) => return Err(ParseHtmlError::new(format!("'DOCTYPE {}' element found in middle of content", t))),
            }
        } else {
            text_content.push(cur_char);
        }
    }
    //Parse HTML until end tag </tag> is found
    return Err(ParseHtmlError::new(format!("End of file without finding tag {}.", tag)));
}


impl HtmlDocument {
    fn new() -> HtmlDocument {
        HtmlDocument {doctype : None, nodes : None}
    }

    fn push_content(&mut self, content : HtmlContent) {
        match &mut self.nodes {
            Some(nodes) => nodes.push(content),
            None => self.nodes = Some(vec![content]),
        }
    }
}

impl FromStr for HtmlDocument {
    type Err = ParseHtmlError;
    fn from_str(html_str: &str) -> std::result::Result<Self, <Self as std::str::FromStr>::Err> {
        let mut doc = HtmlDocument::new();
        let mut chs = html_str.chars();
        let mut buffer = String::new();
        while let Some(ch) = chs.next() {
            if ch == '<' {
                if buffer.len() > 0 {
                    doc.push_content(HtmlContent::Text(buffer));
                    buffer = String::new();
                }
                match parse_html_tag(&mut chs, None)? {
                    HtmlTag::DocType(t) => doc.doctype = Some(t),
                    HtmlTag::NewTag(node_rc) =>  doc.push_content(HtmlContent::Node(node_rc)),
                    HtmlTag::EndTag(t) => return Err(ParseHtmlError::new(format!("Found end tag {} before opening tag.", t))),
                    HtmlTag::Comment(c) => doc.push_content(HtmlContent::Comment(c)),
                }
            } else {
                buffer.push(ch);
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
        let doc = HtmlDocument::from_str(test_html).unwrap();
        assert_eq!(doc.doctype, Some("html".to_owned()));
        let mut meta_attrs = HashMap::new();
        meta_attrs.insert("http-equiv".to_owned(), "Content-Type".to_owned());
        meta_attrs.insert("content".to_owned(), "text/html; charset=UTF-8".to_owned());
        let mut desc_attrs = HashMap::new();
        desc_attrs.insert("name".to_owned(), "description".to_owned());
        desc_attrs.insert("content".to_owned(), "A blank HTML document for testing purposes.".to_owned());
        let mut auth_attrs = HashMap::new();
        auth_attrs.insert("name".to_owned(), "author".to_owned());
        auth_attrs.insert("content".to_owned(), "Six Revisions".to_owned());
        let mut view_attrs = HashMap::new();
        view_attrs.insert("name".to_owned(), "viewport".to_owned());
        view_attrs.insert("content".to_owned(), "width=device-width, initial-scale=1".to_owned());
        let mut link_attrs = HashMap::new();
        link_attrs.insert("rel".to_owned(), "icon".to_owned());
        link_attrs.insert("href".to_owned(), "http://sixrevisions.com/favicon.ico".to_owned());
        link_attrs.insert("type".to_owned(), "image/x-icon".to_owned());
        let mut  a1_attrs = HashMap::new();
        a1_attrs.insert("href".to_owned(), "https://www.webfx.com/blog/images/assets/cdn.sixrevisions.com/0435-01_html5_download_attribute_demo/html5download-demo.html".to_owned());
        let mut a2_attrs = HashMap::new();
        a2_attrs.insert("href".to_owned(), "http://sixrevisions.com/html5/download-attribute/".to_owned());
        let mut parent_node = Rc::new(RefCell::new(HtmlNode {
            tag : "html".to_owned(),
            class : None,
            id : None,
            attributes : None,
            parent : None,
            contents : None,

        }));

        let mut head_node = Rc::new(RefCell::new(HtmlNode {
            tag : "head".to_owned(),
            class : None,
            id : None,
            attributes : None,
            parent : Some(Rc::downgrade(&parent_node)),
            contents : None,
        }));
        head_node.borrow_mut().contents =  Some(vec![
                HtmlContent::Node(Rc::new(RefCell::new(HtmlNode {tag : "meta".to_owned(), id : None, class : None, attributes : Some(meta_attrs), contents : None, parent : Some(Rc::downgrade(&head_node))}))),
                HtmlContent::Text("\n".to_owned()),
                HtmlContent::Node(Rc::new(RefCell::new(HtmlNode { tag : "title".to_owned(), class : None, id : None, attributes : None, contents : Some(vec![HtmlContent::Text("A Sample HTML Document (Test File)".to_owned())]), parent : Some(Rc::downgrade(&head_node))}))),
                HtmlContent::Text("\n\n".to_owned()),
                HtmlContent::Node(Rc::new(RefCell::new(HtmlNode {tag : "meta".to_owned(), id : None, class : None, attributes : Some(desc_attrs), contents : None, parent : Some(Rc::downgrade(&head_node))}))),
                HtmlContent::Text("\n".to_owned()),
                HtmlContent::Node(Rc::new(RefCell::new(HtmlNode {tag : "meta".to_owned(), id : None, class : None, attributes : Some(auth_attrs), contents : None, parent : Some(Rc::downgrade(&head_node))}))),
                HtmlContent::Text("\n".to_owned()),
                HtmlContent::Node(Rc::new(RefCell::new(HtmlNode {tag : "meta".to_owned(), id : None, class : None, attributes : Some(view_attrs), contents : None, parent : Some(Rc::downgrade(&head_node))}))),
                HtmlContent::Text("\n".to_owned()),
                HtmlContent::Node(Rc::new(RefCell::new(HtmlNode {tag : "link".to_owned(), id : None, class : None, attributes : Some(link_attrs), contents : None, parent : Some(Rc::downgrade(&head_node))}))),
                HtmlContent::Text("\n".to_owned()),
            ]);
        let mut body_node = Rc::new(RefCell::new(HtmlNode {
            tag : "body".to_owned(),
            class : None,
            id : None,
            attributes : None,
            parent : Some(Rc::downgrade(&parent_node)),
            contents : None,
        }));
        let stacked_node = Rc::new(RefCell::new(HtmlNode{
            tag : "p".to_owned(),
            id : None, 
            class : None, 
            attributes : None,
            parent : Some(Rc::downgrade(&body_node)),
            contents : None,
        }));
        stacked_node.borrow_mut().contents = Some(vec![
                                HtmlContent::Node(Rc::new(RefCell::new(HtmlNode {tag:"a".to_owned(), id : None, class : None, attributes : Some(a1_attrs), contents : Some(vec![HtmlContent::Text("Go back to the demo".to_owned())]), parent : Some(Rc::downgrade(&stacked_node))}))),
                                ]);
        let stacked_node2 = Rc::new(RefCell::new(HtmlNode{
            tag : "p".to_owned(),
            id : None, 
            class : None, 
            attributes : None,
            parent : Some(Rc::downgrade(&body_node)),
            contents : None,
        }));
        stacked_node2.borrow_mut().contents = Some(vec![
                                HtmlContent::Node(Rc::new(RefCell::new(HtmlNode {tag:"a".to_owned(), id : None, class : None, attributes : Some(a1_attrs), contents : Some(vec![HtmlContent::Text("Read the HTML5 download attribute guide".to_owned())]), parent : Some(Rc::downgrade(&stacked_node2))}))),
                                ]);
        body_node.borrow_mut().contents = Some(vec![
                HtmlContent::Text("\n    \n".to_owned()),
                HtmlContent::Node(Rc::new(RefCell::new(HtmlNode {tag : "h1".to_owned(), id : None, class : Some(vec!["heading".to_owned()]), attributes : None, contents : Some(vec![HtmlContent::Text("A Sample HTML Document (Test File)".to_owned())]), parent : Some(Rc::downgrade(&body_node))}))),
                HtmlContent::Text("\n".to_owned()),
                HtmlContent::Node(Rc::new(RefCell::new(HtmlNode {tag : "p".to_owned(), id : None, class : None, attributes : None, contents : Some(vec![HtmlContent::Text("A blank HTML document for testing purposes.".to_owned())]), parent : Some(Rc::downgrade(&body_node))}))),
                HtmlContent::Text("\n".to_owned()),
                HtmlContent::Node(stacked_node),
                HtmlContent::Text("\n".to_owned()),
                HtmlContent::Node(stacked_node2),
                HtmlContent::Text("\n\n\n".to_owned()),
            ]);
        parent_node.borrow_mut().contents = Some(vec![
            HtmlContent::Node(head_node),
            HtmlContent::Text("\n".to_owned()),
            HtmlContent::Node(body_node),
        ]);
        let nodes = vec![
            HtmlContent::Text("\n".to_owned()),
            HtmlContent::Comment(" saved from url=(0117)https://www.webfx.com/blog/images/assets/cdn.sixrevisions.com/0435-01_html5_download_attribute_demo/samp/htmldoc.html ".to_owned()),
            HtmlContent::Text("\n".to_owned()),
            HtmlContent::Node(parent_node),
        ];


        assert_eq!(doc.nodes, Some(nodes));
        
    }
}

struct HtmlNodeIterator<'a> {
    node : &'a HtmlNode,
    content_vec_iter : Option<std::slice::Iter<'a, HtmlContent>>,
    current_content_iter : Option<Box<HtmlNodeIterator<'a>>>,
}

impl <'a> Iterator for HtmlNodeIterator <'a> {
    type Item = Rc<RefCell<HtmlNode>>;
    fn next(&mut self) -> std::option::Option<<Self as std::iter::Iterator>::Item>
    { 
        match &mut self.content_vec_iter {
            None => {
                //Initialise the vec iter
                match &self.node.contents {
                    None => None, //no content - return None
                    Some(nodes) => {
                        self.content_vec_iter = Some(nodes.iter());
                        self.next() //call next again to go through other path
                    },
                }
            },
            Some(iter) => {
                // check the current node iter then  increment it if needed
                match &mut self.current_content_iter {
                    None => {
                        //get next node from the content then set up the current_node_iter and return the node found
                        while let Some(content) = iter.next() {
                            match content {
                                HtmlContent::Comment(_) => (),
                                HtmlContent::Text(_) => (),
                                HtmlContent::Node(n) => {
                                    self.current_content_iter = Some(Box::new(n.borrow().iter()));
                                    return Some(n.clone());          
                                }
                            }
                        }
                        None
                    },
                    Some(node_iter) => {
                        match node_iter.next() {
                            Some(n) => Some(n),
                            None => {
                                // end of this node, move current_node_iter to next node (via next recursion)
                                self.current_content_iter = None;
                                self.next()
                            },
                        }
                    }
                }

            }
        }
    }
}

impl HtmlNode {
    fn iter(&self) -> HtmlNodeIterator {
        HtmlNodeIterator { node : self, content_vec_iter : None, current_content_iter : None}
    }
}

struct HtmlDocIterator<'a> {
    doc : &'a HtmlDocument,
    node_vec_iter : Option<std::slice::Iter<'a, HtmlContent>>,
    current_node_iter : Option<HtmlNodeIterator<'a>>
}

impl<'a> Iterator for HtmlDocIterator<'a> {
    type Item = Rc<RefCell<HtmlNode>>;
    fn next(&mut self) -> std::option::Option<<Self as std::iter::Iterator>::Item> 
    {
        match &mut self.node_vec_iter {
            None => {
                //Initialise the vec iter
                match &self.doc.nodes {
                    None => None, //no content - return None
                    Some(nodes) => {
                        self.node_vec_iter = Some(nodes.iter());
                        self.next() //call next again to go through other path
                    },
                }
            },
            Some(iter) => {
                // check the current node iter then  increment it if needed
                match &mut self.current_node_iter {
                    None => {
                        //get next node from the content then set up the current_node_iter and return the node found
                        while let Some(content) = iter.next() {
                            match content {
                                HtmlContent::Comment(_) => (),
                                HtmlContent::Text(_) => (),
                                HtmlContent::Node(n) => {
                                    self.current_node_iter = Some(n.borrow().iter());
                                    return Some(n.clone());          
                                }
                            }
                        }
                        None
                    },
                    Some(node_iter) => {
                        match node_iter.next() {
                            Some(n) => Some(n),
                            None => {
                                // end of this node, move current_node_iter to next node (via next recursion)
                                self.current_node_iter = None;
                                self.next()
                            },
                        }
                    }
                }

            }
        }
    }
}

impl HtmlDocument {
    fn iter(&self) -> HtmlDocIterator {
        HtmlDocIterator {doc : self, current_node_iter : None, node_vec_iter : None}
    }
}

struct HtmlContentSliceIterator<'a> {
    slice_iter : std::slice::Iter<'a, HtmlContent>,
    current_content_iter : Option<HtmlNodeIterator<'a>>,
}

trait HtmlContentSliceIterCreator {
    fn html_iter(&self) -> HtmlContentSliceIterator;
}

impl HtmlContentSliceIterCreator for [HtmlContent] {
    fn html_iter(&self) -> HtmlContentSliceIterator {
        HtmlContentSliceIterator { slice_iter : self.iter(), current_content_iter : None, }
    }
}
impl<'a> Iterator for HtmlContentSliceIterator<'a> {
    type Item = Rc<RefCell<HtmlNode>>;
    fn next(&mut self) -> std::option::Option<<Self as std::iter::Iterator>::Item> 
    {
        // check the current node iter then  increment it if needed
        match &mut self.current_content_iter {
            None => {
                //get next node from the content then set up the current_node_iter and return the node found
                while let Some(content) = self.slice_iter.next() {
                    match content {
                        HtmlContent::Comment(_) => (),
                        HtmlContent::Text(_) => (),
                        HtmlContent::Node(n) => {
                            self.current_content_iter = Some(n.borrow().iter());
                            return Some(n.clone());          
                        }
                    }
                }
                None
            },
            Some(node_iter) => {
                match node_iter.next() {
                    Some(n) => Some(n),
                    None => {
                        // end of this node, move current_node_iter to next node (via next recursion)
                        self.current_content_iter = None;
                        self.next()
                    },
                }
            }
        }

    }
}

struct HtmlNodeSliceIterator<'a> {
    slice_iter : std::slice::Iter<'a, Rc<RefCell<HtmlNode>>>,
    current_content_iter : Option<HtmlNodeIterator<'a>>,
}

trait HtmlNodeSliceIterCreator {
    fn html_iter(&self) -> HtmlNodeSliceIterator;
}

impl HtmlNodeSliceIterCreator for [Rc<RefCell<HtmlNode>>] {
    fn html_iter(&self) -> HtmlNodeSliceIterator {
        HtmlNodeSliceIterator { slice_iter : self.iter(), current_content_iter : None, }
    }
}
impl<'a> Iterator for HtmlNodeSliceIterator<'a> {
    type Item = Rc<RefCell<HtmlNode>>;
    fn next(&mut self) -> std::option::Option<<Self as std::iter::Iterator>::Item> 
    {
        // check the current node iter then  increment it if needed
        match &mut self.current_content_iter {
            None => {
                //get next node from the content then set up the current_node_iter and return the node found
                match self.slice_iter.next() {
                    Some(n) => {
                            self.current_content_iter = Some(n.borrow().iter());
                            return Some(n.clone());          
                        },
                    None => None,
                }
            },
            Some(node_iter) => {
                match node_iter.next() {
                    Some(n) => Some(n),
                    None => {
                        // end of this node, move current_node_iter to next node (via next recursion)
                        self.current_content_iter = None;
                        self.next()
                    },
                }
            }
        }

    }
}

impl HtmlNodeSliceIterCreator for Vec<Rc<RefCell<HtmlNode>>> {
    fn html_iter(&self) -> HtmlNodeSliceIterator {
        HtmlNodeSliceIterator { slice_iter : self.iter(), current_content_iter : None, }
    }
}
impl HtmlContentSliceIterCreator for Vec<HtmlContent> {
    fn html_iter(&self) -> HtmlContentSliceIterator {
        HtmlContentSliceIterator { slice_iter : self.iter(), current_content_iter : None, }
    }
}


#[cfg(test)]
mod html_iterator_tests {
    use super::*;

    #[test]
    fn HTMLDocumentIteratorTest() {
        let test_html = r#"<!DOCTYPE html>
<html>
<body>
<h1 class=heading>A Sample HTML Document (Test File)</h1>
<p>A blank HTML document for testing purposes.</p>
<p><a href="a_path">Go back to the demo</a></p>
</body></html>"#;

        let doc = HtmlDocument::from_str(test_html).unwrap();
        assert_eq!(doc.doctype, Some("html".to_owned()));
        let mut meta_attrs = HashMap::new();
        meta_attrs.insert("http-equiv".to_owned(), "Content-Type".to_owned());
        meta_attrs.insert("content".to_owned(), "text/html; charset=UTF-8".to_owned());
        let mut desc_attrs = HashMap::new();
        desc_attrs.insert("name".to_owned(), "description".to_owned());
        desc_attrs.insert("content".to_owned(), "A blank HTML document for testing purposes.".to_owned());
        let mut auth_attrs = HashMap::new();
        auth_attrs.insert("name".to_owned(), "author".to_owned());
        auth_attrs.insert("content".to_owned(), "Six Revisions".to_owned());
        let mut view_attrs = HashMap::new();
        view_attrs.insert("name".to_owned(), "viewport".to_owned());
        view_attrs.insert("content".to_owned(), "width=device-width, initial-scale=1".to_owned());
        let mut link_attrs = HashMap::new();
        link_attrs.insert("rel".to_owned(), "icon".to_owned());
        link_attrs.insert("href".to_owned(), "http://sixrevisions.com/favicon.ico".to_owned());
        link_attrs.insert("type".to_owned(), "image/x-icon".to_owned());
        let mut  a1_attrs = HashMap::new();
        a1_attrs.insert("href".to_owned(), "a_path".to_owned());
        let nodes = vec![
            HtmlContent::Text("\n".to_owned()),
            HtmlContent::Node(HtmlNode{
                tag : "html".to_owned(),
                class : None,
                id : None,
                attributes : None,
                contents : Some(vec![
                    HtmlContent::Text("\n".to_owned()),
                    HtmlContent::Node(HtmlNode{
                        tag : "body".to_owned(),
                        class : None,
                        id : None,
                        attributes : None,
                        contents : Some(vec![
                            HtmlContent::Text("\n".to_owned()),
                            HtmlContent::Node(HtmlNode {tag : "h1".to_owned(), id : None, class : Some(vec!["heading".to_owned()]), attributes : None, contents : Some(vec![HtmlContent::Text("A Sample HTML Document (Test File)".to_owned())])}),
                            HtmlContent::Text("\n".to_owned()),
                            HtmlContent::Node(HtmlNode {tag : "p".to_owned(), id : None, class : None, attributes : None, contents : Some(vec![HtmlContent::Text("A blank HTML document for testing purposes.".to_owned())])}),
                            HtmlContent::Text("\n".to_owned()),
                            HtmlContent::Node(HtmlNode {tag : "p".to_owned(), id : None, class : None, attributes : None, contents : Some(vec![
                                HtmlContent::Node(HtmlNode {tag:"a".to_owned(), id : None, class : None, attributes : Some(a1_attrs.clone()), contents : Some(vec![HtmlContent::Text("Go back to the demo".to_owned())])}),
                                ])}),
                            HtmlContent::Text("\n".to_owned()),
                        ]),
                    }),
                ]),
            })
        ];
        
        let error_node = HtmlNode::new("error");
        let top_node = match &nodes[1] {
            HtmlContent::Node(n) => n,
            HtmlContent::Comment(_) => &error_node,
            HtmlContent::Text(_) => &error_node,
        };
        let next_node = HtmlNode{
                        tag : "body".to_owned(),
                        class : None,
                        id : None,
                        attributes : None,
                        contents : Some(vec![
                            HtmlContent::Text("\n".to_owned()),
                            HtmlContent::Node(HtmlNode {tag : "h1".to_owned(), id : None, class : Some(vec!["heading".to_owned()]), attributes : None, contents : Some(vec![HtmlContent::Text("A Sample HTML Document (Test File)".to_owned())])}),
                            HtmlContent::Text("\n".to_owned()),
                            HtmlContent::Node(HtmlNode {tag : "p".to_owned(), id : None, class : None, attributes : None, contents : Some(vec![HtmlContent::Text("A blank HTML document for testing purposes.".to_owned())])}),
                            HtmlContent::Text("\n".to_owned()),
                            HtmlContent::Node(HtmlNode {tag : "p".to_owned(), id : None, class : None, attributes : None, contents : Some(vec![
                                HtmlContent::Node(HtmlNode {tag:"a".to_owned(), id : None, class : None, attributes : Some(a1_attrs.clone()), contents : Some(vec![HtmlContent::Text("Go back to the demo".to_owned())])}),
                                ])}),
                            HtmlContent::Text("\n".to_owned()),
                        ]),
                    };
        let third_node = HtmlNode {tag : "h1".to_owned(), id : None, class : Some(vec!["heading".to_owned()]), attributes : None, contents : Some(vec![HtmlContent::Text("A Sample HTML Document (Test File)".to_owned())])};
        let forth_node = HtmlNode {tag : "p".to_owned(), id : None, class : None, attributes : None, contents : Some(vec![HtmlContent::Text("A blank HTML document for testing purposes.".to_owned())])};
        let fifth_node = HtmlNode {tag : "p".to_owned(), id : None, class : None, attributes : None, contents : Some(vec![
                                HtmlContent::Node(HtmlNode {tag:"a".to_owned(), id : None, class : None, attributes : Some(a1_attrs.clone()), contents : Some(vec![HtmlContent::Text("Go back to the demo".to_owned())])}),
                                ])};
        let last_node = HtmlNode {tag:"a".to_owned(), id : None, class : None, attributes : Some(a1_attrs.clone()), contents : Some(vec![HtmlContent::Text("Go back to the demo".to_owned())])};
        let mut doc_iter = doc.iter();
        assert_eq!(doc_iter.next(), Some(top_node));
        assert_eq!(doc_iter.next(), Some(&next_node));
        assert_eq!(doc_iter.next(), Some(&third_node));
        assert_eq!(doc_iter.next(), Some(&forth_node));
        assert_eq!(doc_iter.next(), Some(&fifth_node));
        assert_eq!(doc_iter.next(), Some(&last_node));
        assert_eq!(doc_iter.next(), None);

        let content_arr = [
            HtmlContent::Node(third_node.clone()),
            HtmlContent::Node(forth_node.clone()),
            HtmlContent::Node(fifth_node.clone()),
            //last node is contained in fifth node
        ];
        let mut content_slice_iter = content_arr[..].html_iter();
        assert_eq!(content_slice_iter.next(), Some(&third_node));
        assert_eq!(content_slice_iter.next(), Some(&forth_node));
        assert_eq!(content_slice_iter.next(), Some(&fifth_node));
        assert_eq!(content_slice_iter.next(), Some(&last_node));
        assert_eq!(content_slice_iter.next(), None);

        let node_arr = [
            third_node.clone(),
            forth_node.clone(),
            fifth_node.clone(),
            //last node is contained in fifth node
        ];
        let mut node_slice_iter = node_arr[..].html_iter();
        assert_eq!(node_slice_iter.next(), Some(&third_node));
        assert_eq!(node_slice_iter.next(), Some(&forth_node));
        assert_eq!(node_slice_iter.next(), Some(&fifth_node));
        assert_eq!(node_slice_iter.next(), Some(&last_node));
        assert_eq!(node_slice_iter.next(), None);

        let content_vec = vec![
            HtmlContent::Node(third_node.clone()),
            HtmlContent::Node(forth_node.clone()),
            HtmlContent::Node(fifth_node.clone()),
            //last node is contained in fifth node
        ];
        let mut content_slice_iter = content_vec.html_iter();
        assert_eq!(content_slice_iter.next(), Some(&third_node));
        assert_eq!(content_slice_iter.next(), Some(&forth_node));
        assert_eq!(content_slice_iter.next(), Some(&fifth_node));
        assert_eq!(content_slice_iter.next(), Some(&last_node));
        assert_eq!(content_slice_iter.next(), None);

        let node_vec = vec![
            third_node.clone(),
            forth_node.clone(),
            fifth_node.clone(),
            //last node is contained in fifth node
        ];
        let mut node_slice_iter = node_vec.html_iter();
        assert_eq!(node_slice_iter.next(), Some(&third_node));
        assert_eq!(node_slice_iter.next(), Some(&forth_node));
        assert_eq!(node_slice_iter.next(), Some(&fifth_node));
        assert_eq!(node_slice_iter.next(), Some(&last_node));
        assert_eq!(node_slice_iter.next(), None);
        
    }
}

trait HtmlSelectorsTrait {
    //To be implemented on both HtmlNode and HtmlContent
    fn matches<'a, T : Into<&'a str>>(selector : T) -> bool;
    fn has_child<'a, T : Into<&'a str>>(selector : T) -> bool;
    fn has_decendant<'a, T : Into<&'a str>>(selector : T) -> bool;
    fn contains_text<'a, T : Into<&'a str>>(selector : T) -> bool;
    fn child_contains_text<'a, T : Into<&'a str>>(selector : T) -> bool;
    fn decendant_contains_text<'a, T : Into<&'a str>>(selector : T) -> bool;
    fn get_text() -> String;
    fn get_child_text() -> String;
    fn get_decendant_text() -> String;
}

impl HtmlSelectorsTrait for HtmlNode {
    fn matches<'a, T : Into<&'a str>>(selector: T) -> bool { todo!() }
    fn has_child<'a, T : Into<&'a str>>(selector: T) -> bool { todo!() }
    fn has_decendant<'a, T : Into<&'a str>>(selector: T) -> bool { todo!() }
    fn contains_text<'a, T : Into<&'a str>>(selector: T) -> bool { todo!() }
    fn child_contains_text<'a, T : Into<&'a str>>(selector: T) -> bool { todo!() }
    fn decendant_contains_text<'a, T : Into<&'a str>>(selector: T) -> bool { todo!() }
    fn get_text() -> std::string::String { todo!() }
    fn get_child_text() -> std::string::String { todo!() }
    fn get_decendant_text() -> std::string::String { todo!() }
}


//Functions to implment
// .matches(selector) - should this include child selectors (eg div p)...probably 
// .has_child(selector)
// .has_decendant(selector)
// .contains_text(text)
// .child_contains_text(text)
// .decendant_contains_text(text)
// .get_text()
// .get_child_text()
// .get_decendant_text()


//Adapters to implement
// select(selector)
// children(selector)
// decendants(selector)
// has_child(selector)
// has_decendant(selector)
// contains_text(text)
// child_contains_text(text)


// Get all divs which contains a p which then select decendants which are a
// CSS selectors don't support his by default!
// doc.iter().select("div").has_child("p").decendants("a")
// 
// Get all a which are children of p which are decendants of divs with class c1
// Using a CSS selector
// doc.iter().select("div.c1 p>a")
// Or in long form...
// doc.iter().select("div.c1").decendants("p").children("a")