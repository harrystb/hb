use crate::error::ParseHtmlError;
use crate::objects::{
    CSSSelectorItem, CSSSelectorRelationship, CSSSelectorRule, HtmlNode, HtmlTag,
};
use std::collections::VecDeque;

// Read from the iterator until a quoted string or word is found (ignoring leading whitespace) then return the string and the character that ended the string
// Endings of a single word can be whitespace or >
pub fn parse_string(chs: &mut std::str::Chars) -> Result<(String, char), ParseHtmlError> {
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

pub fn parse_until_one_of(
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

pub fn parse_until_char(
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

pub fn parse_until_str(
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

pub fn parse_until<F: Fn(&char) -> bool>(
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

pub fn parse_attibute_value(attr_value: String) -> Vec<String> {
    let mut v = vec![];
    for val in attr_value.split_ascii_whitespace() {
        v.push(val.to_string());
    }
    return v;
}

pub fn get_next_non_whitespace(chs: &mut std::str::Chars) -> Result<char, ParseHtmlError> {
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
pub enum ParsedTagType {
    EndTag(String),  //eg </div>
    NewTag(HtmlTag), //eg <div class="test">
    Comment(String), //eg <!-- text --!>
    DocType(String),
}

pub fn parse_html_tag(chs: &mut std::str::Chars) -> Result<ParsedTagType, ParseHtmlError> {
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

#[cfg(test)]
mod parse_html_document_tests {
    use super::*;
    use crate::objects::HtmlDocument;

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
        let doc_from_str = test_html.parse::<HtmlDocument>().unwrap();
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

pub fn parse_css_selector_rule(selector: &str) -> Result<CSSSelectorRule, ParseHtmlError> {
    let mut chs = selector.chars().peekable();
    todo!();
}

pub fn parse_css_selector_item(
    chs: &mut std::iter::Peekable<std::str::Chars>,
) -> Result<Option<CSSSelectorItem>, ParseHtmlError> {
    //consume whitespace
    loop {
        match chs.peek() {
            None => {
                return Err(ParseHtmlError::with_msg(
                    "Could not parse a CSS selector item - only whitespace in string.",
                ))
            }
            Some(ch) => {
                if ch.is_ascii_whitespace() {
                    chs.next();
                } else {
                    break;
                }
            }
        }
    }

    // read until one of " " + > ~

    let mut selector_item = String::new();
    loop {
        match chs.peek() {
            None => {
                break;
            }
            Some(ch) => {
                if ch == &' ' || ch == &'+' || ch == &'>' || ch == &'~' {
                    break;
                }
                selector_item.push(chs.next().unwrap());
            }
        }
    }

    //parse the selector item
    todo!();
}

fn parse_css_selector_item_seperator(
    chs: &mut std::iter::Peekable<std::str::Chars>,
) -> Result<Option<String>, ParseHtmlError> {
    todo!();
}

//TODO: Doc strings and tests
