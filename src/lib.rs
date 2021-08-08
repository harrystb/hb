use std::collections::VecDeque;
use std::collections::HashMap;
use std::str::FromStr;

struct ParseHtmlError {
    msg : String,
}

impl ParseHtmlError {
    fn new(msg : String) -> ParseHtmlError {
        ParseHtmlError { msg : msg}
    }
}

impl std::fmt::Display for ParseHtmlError {
    fn fmt(&self, f : &mut  std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        write!(f, "Error occured parsing HTML '{}'", self.msg);
        Ok(())
    }
}
impl std::fmt::Debug for ParseHtmlError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        write!(f, "Error occured parsing HTML '{}'", self.msg);
        Ok(())
    }
}

struct HtmlDocument {
    doctype : Option<String>,
    nodes : Option<Vec<HtmlNode>>,
}

#[derive(Debug, PartialEq)]
struct HtmlNode {
    tag : Option<String>,
    contents : Option<Vec<HtmlContent>>,
    class : Option<Vec<String>>,
    id : Option<Vec<String>>,
    attributes : Option<HashMap<String, String>>,
}

#[derive(Debug, PartialEq)]
enum HtmlContent {
    Text(String),
    Node(HtmlNode),
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

impl FromStr for CSSSelector {
    type Err = ParseHtmlError;
    fn from_str(s: &str) -> std::result::Result<Self, <Self as std::str::FromStr>::Err> {
        if s == "*" {
            return Ok(CSSSelector::All);
        }
        let mut output = vec![];
        for word in s.split_ascii_whitespace() {
            if word.starts_with('#') {
                output.push(CSSSelectorData {class: None, id: Some(word[1..].to_string()), tag : None});
            }
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
        if output.len() > 0 {
            return Ok(CSSSelector::Specific(output));
        }
        return Err(ParseHtmlError::new(format!("Unknown selector {}", s)));    
    }
}

#[cfg(test)]
mod CSSSelectorTests {
    use super::*;

    fn CreateSelectorFromStringTest() -> Result<(), ParseHtmlError> {
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
    fn new() -> HtmlNode {
        HtmlNode { tag : None, class : None, id : None, contents : None, attributes : None}
    }

    fn Select(&self, selector : &CSSSelector) -> Option<Vec<&HtmlNode>> {
        let mut out : Vec<&HtmlNode> = Vec::new();

        match selector {
            CSSSelector::All => {
                out.push(self);
                ()
            },
            CSSSelector::Specific(selector_data_vec) =>{
                let mut class_matches = false;
                let mut id_matches = false;
                for selector_data in selector_data_vec {
                    class_matches = match &selector_data.class {
                        None => true,
                        Some(selector_class) => {
                            match &self.class {
                                None => false,
                                Some(classes) => {
                                    let mut passes = false;
                                    for class in classes {
                                        if class == selector_class {
                                            passes = true;
                                            break;
                                        }
                                    }
                                    passes
                                }
                            }
                        }
                    };
                    id_matches = match &selector_data.id {
                        None => true,
                        Some(selector_id) => {
                            match &self.id {
                                None => false,
                                Some(ids) => {
                                    let mut passes = false;
                                    for id in ids {
                                        if id == selector_id {
                                            passes = true;
                                            break;
                                        }
                                    }
                                    passes
                                }
                            }
                        }
                    };
                }
                if class_matches & id_matches {
                    out.push(self);
                }
                ()
            }
        }

        match &self.contents {
            None => (),
            Some(contents) => {
                for content in contents{
                    match content {
                        HtmlContent::Text(_) => (),
                        HtmlContent::Node(node) => {
                            match node.Select(&selector) {
                                None => (),
                                Some(nodes) => {
                                    out.extend(nodes);
                                    ()
                                } 
                            }
                        },
                        HtmlContent::Comment(_) => (),
                    }
                }
            }
        }
        if out.len() > 0 {
            return None;
        }
        Some(out)
    }

}

#[cfg(test)]
mod HtmlNodeTests {
    use super::*;

    #[test]
    fn node_selection() {
        let mut root = HtmlNode::new();
        root.tag = Some("html".to_string());

        let mut node1 = HtmlNode::new();
        node1.tag = Some("div".to_string());
        node1.class = Some(vec!["test".to_string(), "test-2".to_string()]);
        let mut node2 = HtmlNode::new();
        node2.tag = Some("div".to_string());
        node2.class = Some(vec!["test".to_string()]);

        root.contents = Some(vec![HtmlContent::Node(node1), HtmlContent::Text("some_text.omg".to_string()), HtmlContent::Node(node2)]);
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
mod ParseIterTests {
    use super::*;

    #[test]
    fn ParseCharsTest() {
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
    NewTag(HtmlNode), //eg <div class="test">
    Comment(String), //eg <!-- text --!>
    None,
}

fn parse_html_tag(chs : &mut std::str::Chars) -> Result<HtmlTag, ParseHtmlError> {
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
        if buffer.ends_with("--!>") {
            buffer.truncate(buffer.len()-4);
            return Ok(HtmlTag::Comment(buffer));
        }
        buffer.push_str(parse_until_str(chs, &"--!", false)?.as_str());
        return Ok(HtmlTag::Comment(buffer))
    }
    let (tag_str, ending) = buffer.split_at(buffer.len()-1);
    let mut node = HtmlNode::new();
    node.tag = Some(tag_str.to_owned());
    if ending == ">" {
        return Ok(HtmlTag::NewTag(node));
    }
    //there are some attributes - parse the attributes

    //define the some checking closures
    let is_ws_eq_or_gt = |ch : &char| -> bool {
        return ch.is_ascii_whitespace() || *ch == '=' || *ch == '>';
    };
    let is_ws_or_gt = |ch : &char| -> bool {
        return ch.is_ascii_whitespace() || *ch == '>';
    };
    loop {
        buffer.clear();
        buffer.push(get_next_non_whitespace(chs)?);
        if buffer == ">" {
            //didn't get an attribute - just got the end of tag symbol
            return Ok(HtmlTag::NewTag(node));
        }
        buffer.push_str(parse_until(chs, is_ws_eq_or_gt, true)?.as_str());
        let (attr_str, attr_ending) = buffer.split_at(buffer.len()-1);
        if attr_ending == ">" {
            if attr_str.len() > 0 {
                return Err(ParseHtmlError::new(format!("Expected value for attribute '{}', got {} instead of '='.", attr_str, attr_ending)));
            }
            //didn't get an attribute - just got the end of tag symbol
            return Ok(HtmlTag::NewTag(node));
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
            return Ok(HtmlTag::NewTag(node));
        }
    }
}
#[cfg(test)]
mod ParseHtmlTagTests {
    use super::*;

    #[test]
    fn ParseHtmlTagTest() {
        assert_eq!(parse_html_tag(&mut "div>".chars()).unwrap(), HtmlTag::NewTag(HtmlNode { tag: Some("div".to_string()), class : None, id : None, attributes : None, contents : None }));
        assert_eq!(parse_html_tag(&mut "/div>".chars()).unwrap(), HtmlTag::EndTag("div".to_string()));
        assert_eq!(parse_html_tag(&mut "/div >".chars()).unwrap(), HtmlTag::EndTag("div".to_string()));
        assert_eq!(
            parse_html_tag(&mut "div  class=\"class1\">".chars()).unwrap(), 
            HtmlTag::NewTag(HtmlNode { tag: Some("div".to_string()), class : Some(vec!["class1".to_string()]), id : None, attributes : None, contents : None })
        );
        assert_eq!(
            parse_html_tag(&mut "div  class=\"class1\" >".chars()).unwrap(), 
            HtmlTag::NewTag(HtmlNode { tag: Some("div".to_string()), class : Some(vec!["class1".to_string()]), id : None, attributes : None, contents : None })
        );
        assert_eq!(
            parse_html_tag(&mut "a class=\"class1\" id = \"id1\">".chars()).unwrap(), 
            HtmlTag::NewTag(HtmlNode { tag: Some("a".to_string()), class : Some(vec!["class1".to_string()]), id : Some(vec!["id1".to_string()]), attributes : None, contents : None })
        );
        assert_eq!(
            parse_html_tag(&mut "a class=\"class1\" id = \"id1\" >".chars()).unwrap(), 
            HtmlTag::NewTag(HtmlNode { tag: Some("a".to_string()), class : Some(vec!["class1".to_string()]), id : Some(vec!["id1".to_string()]), attributes : None, contents : None })
        );
        let mut map = HashMap::new();
        map.insert("other_attr".to_string(), "something".to_string());
        assert_eq!(
            parse_html_tag(&mut "a class=\"class1\" other_attr=something>".chars()).unwrap(), 
            HtmlTag::NewTag(HtmlNode { tag: Some("a".to_string()), class : Some(vec!["class1".to_string()]), id : None, attributes : Some(map), contents : None })
        );
        assert_eq!(
            parse_html_tag(&mut "!-- something --!>".chars()).unwrap(), 
            HtmlTag::Comment(" something ".to_string())
        );
        assert_eq!(
            parse_html_tag(&mut "!-- something\n something else --!>".chars()).unwrap(), 
            HtmlTag::Comment(" something\n something else ".to_string())
        );
        //TODO: How to test failing cases...
        //TODO: split tests into more functions (eg start tag, end tag, comment, etc.)
    }
}

fn ParseHtmlContent(chs : &mut std::str::Chars, tag : &str) -> Result<Option<Vec<HtmlContent>>, ParseHtmlError> {
    let mut text_content = String::new();
    let mut content : Vec<HtmlContent> = Vec::new();
    while let Some(cur_char) = chs.next() {
        if cur_char == '<' {
            if text_content.len() > 0 {
                content.push(HtmlContent::Text(text_content));
                text_content = String::new();
            }
            //Read rest of tag - passing along any errors that were encountered.
            match parse_html_tag(chs)? {
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
                HtmlTag::NewTag(node) => {
                    content.push(HtmlContent::Node(node));
                },
                HtmlTag::Comment(comment) => {
                    content.push(HtmlContent::Comment(comment));
                },
                HtmlTag::None => (),
            }
        } else {
            text_content.push(cur_char);
        }
    }
    //Parse HTML until end tag </tag> is found
    return Err(ParseHtmlError::new(format!("End of file without finding tag {}.", tag)));
}


enum ParseHtmlMode {
    InStartTag,
    InEndTag,
    InContent,
}

impl FromStr for HtmlDocument {
    type Err = ParseHtmlError;
    fn from_str(html_str: &str) -> std::result::Result<Self, <Self as std::str::FromStr>::Err> {
        let mut parse_mode = ParseHtmlMode::InContent;
        let mut cur_slice : &str;
        let mut node = HtmlNode::new();
        let ch_iter = html_str.chars();
        for ch in ch_iter {
            if ch == '<' {
                match parse_mode {
                    ParseHtmlMode::InStartTag => return Err(ParseHtmlError::new("".to_string())),
                    ParseHtmlMode::InEndTag => return Err(ParseHtmlError::new("".to_string())),
                    ParseHtmlMode::InContent => {
                        //Need to work out how to determine it should be start or end tag...
                        //TODO: Follow idea in notebook - parse_until....
                        parse_mode = ParseHtmlMode::InStartTag;
                    }
                }
            }
        }
        Err(ParseHtmlError::new("".to_string()))
    }
}