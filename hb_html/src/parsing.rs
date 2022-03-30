use crate::error::ParseHtmlError;
use crate::objects::{
    CssAttributeCompareType, CssRefiner, CssRefinerNumberType, CssSelector, CssSelectorItem,
    CssSelectorRelationship, CssSelectorRule, HtmlNode, HtmlTag,
};
use std::collections::VecDeque;
use std::str::FromStr;

// Read from the iterator until a quoted string or word is found (ignoring leading whitespace) then return the string and the character that ended the string
// Endings of a single word can be whitespace or >
pub fn parse_string(chs: &mut std::str::Chars) -> Result<(String, char), ParseHtmlError> {
    let mut buffer = String::with_capacity(50);
    let mut is_quoted = false;
    //consume leading whitespace
    let ch = get_next_non_whitespace(chs).map_err(|e| e.add_context("could not get string"))?;
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
    if is_quoted {
        return Err(ParseHtmlError::new(format!(
            "Closing '\"' for string '{}' not found",
            buffer
        )));
    }
    if buffer.len() == 0 {
        return Err(ParseHtmlError::with_msg("No chars found."));
    }
    Ok((buffer, ' '))
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
        "end of string '{}' encountered before any end char '{:?}' was found",
        buffer, end_chars
    )));
}

pub fn parse_until_end_or_one_of_peekable(
    chs: &mut std::iter::Peekable<std::str::Chars>,
    end_chars: Vec<char>,
) -> Option<String> {
    let mut buffer = String::new();
    while let Some(ch) = chs.peek() {
        for end_char in &end_chars {
            if ch == end_char {
                return Some(buffer);
            }
        }
        buffer.push(chs.next().unwrap());
    }
    if buffer.len() > 0 {
        return Some(buffer);
    }
    None
}

pub fn parse_until_one_of_peekable(
    chs: &mut std::iter::Peekable<std::str::Chars>,
    end_chars: Vec<char>,
) -> Result<String, ParseHtmlError> {
    let mut buffer = String::new();
    while let Some(ch) = chs.peek() {
        for end_char in &end_chars {
            if ch == end_char {
                return Ok(buffer);
            }
        }
        buffer.push(chs.next().unwrap());
    }
    return Err(ParseHtmlError::new(format!(
        "end of string '{}' encountered before any end char '{:?}' was found",
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
        "end of string '{}' encountered before end char '{}' was found",
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
        "end of string '{}{}' encountered before any end string '{}' was found",
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
        "end of string encountered without terminating character in string '{}'",
        buffer
    )));
}

pub fn parse_until_and_including_char(
    chs: &mut std::iter::Peekable<std::str::Chars>,
    ending: char,
) -> Result<String, ParseHtmlError> {
    let mut buffer = String::new();
    while let Some(ch) = chs.peek() {
        if ch == &ending {
            buffer.push(chs.next().unwrap());
            return Ok(buffer);
        }
        buffer.push(chs.next().unwrap());
    }
    return Err(ParseHtmlError::new(format!(
        "end of string '{}' encountered before ending '{}' was found",
        buffer, ending
    )));
}

pub fn parse_until_char_peekable(
    chs: &mut std::iter::Peekable<std::str::Chars>,
    ending: char,
) -> Result<String, ParseHtmlError> {
    let mut buffer = String::new();
    while let Some(ch) = chs.peek() {
        if ch == &ending {
            return Ok(buffer);
        }
        buffer.push(chs.next().unwrap());
    }
    return Err(ParseHtmlError::new(format!(
        "end of string '{}' encountered before ending '{}' was found",
        buffer, ending
    )));
}

pub fn parse_contents_of_braces(
    chs: &mut std::iter::Peekable<std::str::Chars>,
) -> Result<String, ParseHtmlError> {
    let mut buffer = String::new();
    let mut level = 0;
    match chs.peek() {
        None => return Err(ParseHtmlError::new(format!("no characters found",))),
        Some(c) => {
            if *c != '(' {
                return Err(ParseHtmlError::new(format!("no opening brace was found")));
            }
        }
    }
    while let Some(ch) = chs.peek() {
        match *ch {
            '(' => {
                level += 1;
                chs.next();
            }
            ')' => {
                level -= 1;
                chs.next();
                if level == 0 {
                    return Ok(buffer);
                }
            }
            _ => buffer.push(chs.next().unwrap()),
        }
    }
    return Err(ParseHtmlError::new(format!(
        "end of string '{}' encountered before closing brace ')' was found",
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
                    parse_until_char(chs, '>', false)
                        .map_err(|e| e.add_context(format!("Could not parse end tag")))?
                        .trim_end()
                        .to_owned(),
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
    buffer.push_str(
        parse_until_one_of(chs, vec![' ', '>'], true)
            .map_err(|e| e.add_context(format!("Could parse starting tag after {}", buffer)))?
            .as_str(),
    );
    if buffer.starts_with("!--") {
        // comment - parse the rest of the comment
        buffer.drain(0..3);
        if buffer.ends_with("-->") {
            buffer.truncate(buffer.len() - 4);
            return Ok(ParsedTagType::Comment(buffer));
        }
        buffer.push_str(
            parse_until_str(chs, &"-->", false)
                .map_err(|e| e.add_context("Could not parse comment tag"))?
                .as_str(),
        );
        return Ok(ParsedTagType::Comment(buffer));
    } else if buffer.starts_with("!DOCTYPE ") {
        buffer.clear();
        return Ok(ParsedTagType::DocType(
            parse_until_char(chs, '>', false)
                .map_err(|e| e.add_context("Could not parse DOCTYPE"))?,
        ));
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
            buffer.push(get_next_non_whitespace(chs).map_err(|e| {
                e.add_context(format!(
                    "Could not get next attribute or '>' for node {}",
                    node
                ))
            })?);
            if buffer == ">" {
                //didn't get an attribute - just got the end of tag symbol
                break;
            }
            buffer.push_str(
                parse_until(chs, is_ws_eq_or_gt, true)
                    .map_err(|e| {
                        e.add_context(format!("Could not get find end of attribute '{}'", buffer))
                    })?
                    .as_str(),
            );
            let (attr_str, attr_ending) = buffer.split_at(buffer.len() - 1);
            if attr_ending == ">" {
                if attr_str.len() > 0 {
                    return Err(ParseHtmlError::new(format!(
                        "Expected value for attribute '{}', got '{}' instead of '='.",
                        attr_str, attr_ending
                    )));
                }
                //didn't get an attribute - just got the end of tag symbol
                break;
            }
            if attr_ending.chars().next().unwrap().is_ascii_whitespace() {
                //clear all whitespace until we get a '='
                let ch = get_next_non_whitespace(chs).map_err(|e| {
                    e.add_context(format!("could not get '=' after attribute '{}'", attr_str))
                })?;
                if ch != '=' {
                    return Err(ParseHtmlError::new(format!(
                        "Expected value for attribute '{}', got '{}' instead of '='.",
                        attr_str, ch
                    )));
                }
            }
            //We have 'attr =' now need to read in the value
            let (attr_value_string, attr_value_ending) = parse_string(chs).map_err(|e| {
                e.add_context(format!("could not get value of attribute '{}'", attr_str))
            })?;
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

    #[test]
    fn parse_html_tag_errors() {
        assert_eq!(
            parse_html_tag(&mut "div".chars()),
            Err(ParseHtmlError::with_msg("Could parse starting tag after d because end of string 'iv' encountered before any end char '[' ', '>']' was found"))
        );
        assert_eq!(
            parse_html_tag(&mut "div class=c1".chars()),
            Err(ParseHtmlError::with_msg("Could not get next attribute or '>' for node <Tag: div , IDs: [], Classes: [\"c1\"], Attributes: {}, Contents: []> because End found while consuming whitespace."))
        );
        assert_eq!(
            parse_html_tag(&mut "div class   ".chars()),
            Err(ParseHtmlError::with_msg("could not get '=' after attribute 'class' because End found while consuming whitespace."))
        );
        assert_eq!(
            parse_html_tag(&mut "div class".chars()),
            Err(ParseHtmlError::with_msg("Could not get find end of attribute 'c' because end of string encountered without terminating character in string 'lass'"))
        );
        assert_eq!(
            parse_html_tag(&mut "div class id  ".chars()),
            Err(ParseHtmlError::with_msg(
                "Expected value for attribute 'class', got 'i' instead of '='."
            ))
        );
        assert_eq!(
            parse_html_tag(&mut "div class =  ".chars()),
            Err(ParseHtmlError::with_msg("could not get value of attribute 'class' because could not get string because End found while consuming whitespace."))
        );
        assert_eq!(
            parse_html_tag(&mut "/div".chars()),
            Err(ParseHtmlError::with_msg("Could not parse end tag because end of string 'div' encountered before end char '>' was found"))
        );
        assert_eq!(
            parse_html_tag(&mut "!-- div".chars()),
            Err(ParseHtmlError::with_msg("Could not parse comment tag because end of string 'div' encountered before any end string '-->' was found"))
        );
        assert_eq!(
            parse_html_tag(&mut "!DOCTYPE".chars()),
            Err(ParseHtmlError::with_msg("Could parse starting tag after ! because end of string 'DOCTYPE' encountered before any end char '[' ', '>']' was found"))
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

#[derive(PartialEq, Debug)]
enum CssSelectorRelationshipType {
    Current,
    Parent,
    Ancestor,
    PreviousSibling,
    PreviousSiblingOnce,
}

pub fn parse_css_selector_rule(selector_rule: &str) -> Result<CssSelectorRule, ParseHtmlError> {
    let mut css_rule = CssSelectorRule::new();
    let mut current_relationship = CssSelectorRelationshipType::Current;
    let mut selector_chs = selector_rule.chars().peekable();
    loop {
        match selector_chs.peek() {
            Some(c) => match c {
                ',' => {
                    return Err(ParseHtmlError::with_msg(format!(
                        "unexpected ',' in css selector rule {}",
                        selector_rule
                    )))
                }
                _ => (),
            },
            None => break,
        }
        let item = match parse_css_selector_item(&mut selector_chs)? {
            //check following characters to work out what the relationship should be
            Some(item) => match parse_css_selector_relationship(&mut selector_chs)? {
                CssSelectorRelationshipType::Current => {
                    css_rule.rules.push(CssSelectorRelationship::Current(item))
                }
                CssSelectorRelationshipType::Ancestor => {
                    css_rule.rules.push(CssSelectorRelationship::Ancestor(item))
                }
                CssSelectorRelationshipType::Parent => {
                    css_rule.rules.push(CssSelectorRelationship::Parent(item))
                }
                CssSelectorRelationshipType::PreviousSibling => css_rule
                    .rules
                    .push(CssSelectorRelationship::PreviousSibling(item)),
                CssSelectorRelationshipType::PreviousSiblingOnce => css_rule
                    .rules
                    .push(CssSelectorRelationship::PreviousSiblingOnce(item)),
            },

            None => break,
        };
    }
    Ok(css_rule)
}

fn parse_css_selector_relationship(
    chs: &mut std::iter::Peekable<std::str::Chars>,
) -> Result<CssSelectorRelationshipType, ParseHtmlError> {
    let mut rel = CssSelectorRelationshipType::Current;
    while let Some(c) = chs.peek() {
        match *c {
            ' ' => {
                if rel == CssSelectorRelationshipType::Current {
                    rel = CssSelectorRelationshipType::Ancestor;
                }
                chs.next();
            }
            '>' => {
                //if we have already read a > or a ~
                if rel != CssSelectorRelationshipType::Ancestor
                    && rel != CssSelectorRelationshipType::Current
                {
                    return Err(ParseHtmlError::with_msg(format!(
                        "found multiple relationship seperators in selector first {:?} and now {:?}",
                        rel,
                        CssSelectorRelationshipType::Parent
                    )));
                }
                chs.next();
                rel = CssSelectorRelationshipType::Parent;
            }
            '~' => {
                //if we have already read a > or a ~
                if rel != CssSelectorRelationshipType::Ancestor
                    && rel != CssSelectorRelationshipType::Current
                {
                    return Err(ParseHtmlError::with_msg(format!(
                        "found multiple relationship seperators in selector first {:?} and now {:?}",
                        rel,
                        CssSelectorRelationshipType::PreviousSibling
                    )));
                }
                chs.next();
                rel = CssSelectorRelationshipType::PreviousSibling;
            }
            '+' => {
                //if we have already read a > or a ~
                if rel != CssSelectorRelationshipType::Ancestor
                    && rel != CssSelectorRelationshipType::Current
                {
                    return Err(ParseHtmlError::with_msg(format!(
                        "found multiple relationship seperators in selector first {:?} and now {:?}",
                        rel,
                        CssSelectorRelationshipType::PreviousSiblingOnce
                    )));
                }
                chs.next();
                rel = CssSelectorRelationshipType::PreviousSiblingOnce;
            }
            _ => break,
        }
    }
    Ok(rel)
}

pub fn parse_css_selector_item(
    chs: &mut std::iter::Peekable<std::str::Chars>,
) -> Result<Option<CssSelectorItem>, ParseHtmlError> {
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
    let mut item_str = String::new();
    let mut level = 0; //for handling ( and )
    loop {
        match chs.peek() {
            None => {
                break;
            }
            Some(ch) => {
                if *ch == '(' {
                    level = level + 1;
                } else if *ch == '(' {
                    level = level - 1;
                } else if (*ch == ' ' || *ch == '+' || *ch == '>' || *ch == '~') && (level == 0) {
                    break;
                }
                item_str.push(chs.next().unwrap());
            }
        }
    }

    if item_str.len() == 0 {
        return Ok(None);
    }

    //parse the selector item
    let mut item_chars = item_str.chars().peekable();
    // check prefix:
    let mut item = CssSelectorItem::new();
    loop {
        match item_chars.peek() {
            None => break,
            Some(c) => match c {
                '.' => {
                    item_chars.next(); //consume the .
                    match parse_until_end_or_one_of_peekable(
                        &mut item_chars,
                        vec!['.', '#', ':', '['],
                    ) {
                        Some(class) => match &mut item.classes {
                            Some(classes) => classes.push(class),
                            None => item.classes = Some(vec![class]),
                        },
                        None => (),
                    }
                }
                '#' => {
                    item_chars.next(); //consume the #
                    match parse_until_end_or_one_of_peekable(
                        &mut item_chars,
                        vec!['.', '#', ':', '['],
                    ) {
                        Some(id) => match &mut item.ids {
                            Some(ids) => ids.push(id),
                            None => item.ids = Some(vec![id]),
                        },
                        None => (),
                    }
                }
                ':' => {
                    item_chars.next(); //consume the :
                    let refiner = parse_css_refiner(&mut item_chars)?;
                    match &mut item.refiners {
                        Some(refiners) => refiners.push(refiner),
                        None => item.refiners = Some(vec![refiner]),
                    }
                }
                '[' => {
                    item_chars.next(); //consume the .
                    match &mut item.attributes {
                        None => {
                            let mut attributes = vec![];
                            attributes.push(parse_css_attribute_rule(&mut item_chars)?);
                            item.attributes = Some(attributes);
                        }
                        Some(attributes) => {
                            attributes.push(parse_css_attribute_rule(&mut item_chars)?)
                        }
                    }
                }
                _ => {
                    match parse_until_end_or_one_of_peekable(
                        &mut item_chars,
                        vec!['.', '#', ':', '['],
                    ) {
                        Some(tag) => item.tag = Some(tag),
                        None => (),
                    }
                } //tag
            },
        }
    }
    Ok(Some(item))
}

/// Parses a peekable chars iterator for a CSS selector attribute rule.
/// CSS selector attribute rules refers to the modifiers in a CSS selector that are
/// contained in square brackets "[]", for example [attr=value].
///
/// # Example
///
/// ```ignore
/// use hb_html::parsing::parse_css_attribute_rule;
/// use hb_html::objects::CssAttributeCompareType;
/// let css_attr_rule = parse_css_attribute_rule("[attr=val]".chars().peekable()).expect();
/// assert_eq!(css_attr_rule, CssAttributeCompareType::Equals(("attr".to_owned(), "val".to_owned())));
/// ```
fn parse_css_attribute_rule(
    chs: &mut std::iter::Peekable<std::str::Chars>,
) -> Result<CssAttributeCompareType, ParseHtmlError> {
    let attr = parse_until_one_of_peekable(chs, vec![']', '=', '|', '^', '$', '*', '~'])?;
    let mut sep = String::new();
    match chs.peek() {
        None => {
            return Err(ParseHtmlError::with_msg(
                "No attribute rule found in between [].",
            ));
        }
        Some(c) => match c {
            ']' => {
                chs.next(); //consume ]
                return Ok(CssAttributeCompareType::Present(attr));
            }
            _ => sep.push(chs.next().unwrap()),
        },
    }
    if sep != "=" {
        match chs.peek() {
            None => {
                return Err(ParseHtmlError::with_msg(format!(
                    "Attribute rule not finished {}{}.",
                    attr, sep
                )));
            }
            Some(c) => {
                if *c != '=' {
                    return Err(ParseHtmlError::with_msg(format!(
                        "unknown attribute rule qualifier {}{}.",
                        sep, c
                    )));
                }
                sep.push(chs.next().unwrap());
            }
        }
    }
    let value = parse_until_char_peekable(chs, ']')?;
    chs.next(); // consume ]
    match sep.as_str() {
        "=" => Ok(CssAttributeCompareType::Equals((attr, value))),
        "|=" => Ok(CssAttributeCompareType::EqualsOrBeingsWith((attr, value))),
        "^=" => Ok(CssAttributeCompareType::BeginsWith((attr, value))),
        "$=" => Ok(CssAttributeCompareType::EndsWith((attr, value))),
        "*=" => Ok(CssAttributeCompareType::Contains((attr, value))),
        "~=" => Ok(CssAttributeCompareType::ContainsWord((attr, value))),
        _ => {
            return Err(ParseHtmlError::with_msg(format!(
                "unknown attribute rule qualifier {}.",
                sep
            )));
        }
    }
}

/// Parses a peekable chars iterator for a CSS selector refiner.
/// CSS selector refiners are refering to the modifiers in a CSS selector that follow a ":",
/// this includes things such a ":first-of-type".
///
/// # Example
///
/// ```ignore
/// use hb_html::parsing::parse_css_refiner;
/// use hb_html::objects::CssRefiner;
/// let css_ref = parse_css_refiner(":first-of-type".chars().peekable()).expect();
/// assert_eq!(css_ref, CssRefiner::FirstOfType);
/// ```
fn parse_css_refiner(
    chs: &mut std::iter::Peekable<std::str::Chars>,
) -> Result<CssRefiner, ParseHtmlError> {
    let refiner = match parse_until_end_or_one_of_peekable(chs, vec!['.', '#', ':', '[', '(']) {
        None => return Err(ParseHtmlError::with_msg("No refiner found after :")),
        Some(r) => r,
    };
    if refiner == "checked" {
        return Ok(CssRefiner::Checked);
    } else if refiner == "default" {
        return Ok(CssRefiner::Default);
    } else if refiner == "disabled" {
        return Ok(CssRefiner::Disabled);
    } else if refiner == "enabled" {
        return Ok(CssRefiner::Enabled);
    } else if refiner == "optional" {
        return Ok(CssRefiner::Optional);
    } else if refiner == "required" {
        return Ok(CssRefiner::Required);
    } else if refiner == "read-only" {
        return Ok(CssRefiner::ReadOnly);
    } else if refiner == "read-write" {
        return Ok(CssRefiner::ReadWrite);
    } else if refiner == "empty" {
        return Ok(CssRefiner::Empty);
    } else if refiner == "first-child" {
        return Ok(CssRefiner::FirstChild);
    } else if refiner == "last-child" {
        return Ok(CssRefiner::LastChild);
    } else if refiner.starts_with("nth-child") {
        return Ok(CssRefiner::NthChild(parse_css_refiner_number(
            &parse_until_and_including_char(chs, ')').map_err(|e| {
                e.add_context(format!(
                    "error while trying to read CSS refiner number after {}",
                    refiner
                ))
            })?,
        )?));
    } else if refiner.starts_with("nth-last-child") {
        return Ok(CssRefiner::NthLastChild(parse_css_refiner_number(
            &parse_until_and_including_char(chs, ')').map_err(|e| {
                e.add_context(format!(
                    "error while trying to read CSS refiner number after {}",
                    refiner
                ))
            })?,
        )?));
    } else if refiner == "only-child" {
        return Ok(CssRefiner::OnlyChild);
    } else if refiner == "first-of-type" {
        return Ok(CssRefiner::FirstOfType);
    } else if refiner == "last-of-type" {
        return Ok(CssRefiner::LastOfType);
    } else if refiner.starts_with("nth-of-type") {
        return Ok(CssRefiner::NthOfType(parse_css_refiner_number(
            &parse_until_and_including_char(chs, ')').map_err(|e| {
                e.add_context(format!(
                    "error while trying to read CSS refiner number after {}",
                    refiner
                ))
            })?,
        )?));
    } else if refiner.starts_with("nth-last-of-type") {
        return Ok(CssRefiner::NthLastOfType(
            parse_css_refiner_number(&parse_until_and_including_char(chs, ')').map_err(|e| {
                e.add_context(format!(
                    "error while trying to read CSS refiner number after {}",
                    refiner
                ))
            })?)
            .map_err(|e| {
                e.add_context(format!(
                    "error while trying to read CSS refiner number after {}",
                    refiner
                ))
            })?,
        ));
    } else if refiner == "only-of-type" {
        return Ok(CssRefiner::OnlyOfType);
    } else if refiner == "not" {
        return Ok(CssRefiner::Not(
            CssSelector::from_str(
                parse_contents_of_braces(chs)
                    .map_err(|e| e.add_context("could not find closing brace for :not( refiner"))?
                    .as_str(),
            )
            .map_err(|e| {
                e.add_context("could not parse css selector inside the :not(..) refiner")
            })?,
        ));
    } else if refiner == "root" {
        return Ok(CssRefiner::Root);
    }
    return Err(ParseHtmlError::with_msg(format!(
        "unknown refiner type {}.",
        refiner
    )));
}

/// Parses a peekable chars iterator for a number or function used in a CSS selector refiner.
/// CSS selector refiners are refering to the modifiers in a CSS selector that follow a ":".
/// The number or function that this parses is used in specific refiners such as ":nth-of-type".
///
/// # Example
///
/// ```ignore
/// use hb_html::parsing::parse_css_refiner_number;
/// use hb_html::objects::CssRefinerNumberType;
/// let css_ref_num = parse_css_refiner_number("(2n+1)".chars().peekable()).expect();
/// assert_eq!(css_ref_num, CssRefinerNumberType::Functional((2,1)));
/// ```
fn parse_css_refiner_number(raw_str: &str) -> Result<CssRefinerNumberType, ParseHtmlError> {
    let mut str_iter = raw_str.chars();
    match str_iter.next() {
        None => {
            return Err(ParseHtmlError::with_msg(format!(
                "No number found for refiner, expected a ( at the start of {}.",
                raw_str
            )))
        }
        Some(c) => {
            if c != '(' {
                return Err(ParseHtmlError::with_msg(format!(
                    "Expected ( after refiner which needs a number/even/odd/function but got {}",
                    c
                )));
            }
        }
    }
    match str_iter.last() {
        None => {
            return Err(ParseHtmlError::with_msg(format!(
                "No ) found after ( in refiner {}.",
                raw_str
            )))
        }
        Some(c) => {
            if c != ')' {
                return Err(ParseHtmlError::with_msg(format!(
                    "Expected ) at end of refiner number/even/odd/function but got {}",
                    c
                )));
            }
        }
    }

    let num_str = &raw_str[1..raw_str.len() - 1];
    let parts: Vec<&str> = num_str.split('+').map(|x| x.trim()).collect();
    if parts.len() > 2 {
        return Err(ParseHtmlError::with_msg(format!(
            "too many +'s present in refiner number {}",
            raw_str
        )));
    }

    if parts.len() == 1 {
        match parts[0] {
            "odd" => return Ok(CssRefinerNumberType::Odd),
            "even" => return Ok(CssRefinerNumberType::Even),
            p => match p.parse::<usize>() {
                Err(_) => {
                    return Err(ParseHtmlError::with_msg(format!(
                        "could not parse number in refiner {}",
                        raw_str
                    )))
                }
                Ok(i) => return Ok(CssRefinerNumberType::Specific(i)),
            },
        }
    }

    if parts[0].chars().last().unwrap() != 'n' {
        return Err(ParseHtmlError::with_msg(format!(
            "error parsing functional refiner, expected a 'n' at the end of {}",
            parts[0]
        )));
    }

    let multi = match parts[0][0..parts[0].len() - 1].parse::<i32>() {
        Err(_) => {
            return Err(ParseHtmlError::with_msg(format!(
                "could not parse int before the n in {}",
                parts[0]
            )))
        }
        Ok(i) => i,
    };
    let b = match parts[1].parse::<i32>() {
        Err(_) => {
            return Err(ParseHtmlError::with_msg(format!(
                "could not parse int in {}",
                parts[1]
            )))
        }
        Ok(i) => i,
    };

    Ok(CssRefinerNumberType::Functional((multi, b)))
}

#[cfg(test)]
mod parse_css_selector_tests {
    use super::*;

    #[test]
    fn parse_css_refiner_test() {
        let tests = vec![
            ("checked", CssRefiner::Checked),
            ("default", CssRefiner::Default),
            ("disabled", CssRefiner::Disabled),
            ("enabled", CssRefiner::Enabled),
            ("optional", CssRefiner::Optional),
            ("required", CssRefiner::Required),
            ("read-only", CssRefiner::ReadOnly),
            ("read-write", CssRefiner::ReadWrite),
            ("empty", CssRefiner::Empty),
            ("first-child", CssRefiner::FirstChild),
            ("last-child", CssRefiner::LastChild),
            (
                "nth-child(1)",
                CssRefiner::NthChild(CssRefinerNumberType::Specific(1)),
            ),
            (
                "nth-child(2)",
                CssRefiner::NthChild(CssRefinerNumberType::Specific(2)),
            ),
            (
                "nth-child(2n+1)",
                CssRefiner::NthChild(CssRefinerNumberType::Functional((2, 1))),
            ),
            (
                "nth-child(odd)",
                CssRefiner::NthChild(CssRefinerNumberType::Odd),
            ),
            (
                "nth-child(even)",
                CssRefiner::NthChild(CssRefinerNumberType::Even),
            ),
            (
                "nth-last-child(1)",
                CssRefiner::NthLastChild(CssRefinerNumberType::Specific(1)),
            ),
            ("only-child", CssRefiner::OnlyChild),
            ("first-of-type", CssRefiner::FirstOfType),
            ("last-of-type", CssRefiner::LastOfType),
            (
                "nth-of-type(1)",
                CssRefiner::NthOfType(CssRefinerNumberType::Specific(1)),
            ),
            (
                "nth-last-of-type(1)",
                CssRefiner::NthLastOfType(CssRefinerNumberType::Specific(1)),
            ),
            ("only-of-type", CssRefiner::OnlyOfType),
            ("root", CssRefiner::Root),
            ("empty:checked", CssRefiner::Empty),
            ("empty[attr]", CssRefiner::Empty),
            ("empty#id", CssRefiner::Empty),
            (
                "not(p#id)",
                CssRefiner::Not(CssSelector::Specific(vec![CssSelectorRule {
                    rules: vec![CssSelectorRelationship::Current(CssSelectorItem {
                        tag: Some("p".to_owned()),
                        classes: None,
                        ids: Some(vec!["id".to_owned()]),
                        refiners: None,
                        attributes: None,
                    })],
                }])),
            ),
        ];

        for t in tests {
            assert_eq!(parse_css_refiner(&mut t.0.chars().peekable()).unwrap(), t.1);
        }
    }

    #[test]
    fn parse_css_refiner_errors_test() {
        let tests = vec![(
            "nth-last-of-type(1a)",
            ParseHtmlError::with_msg(
                "error while trying to read CSS refiner number after nth-last-of-type because could not parse number in refiner (1a)",
            )),
            ("nth-last-of-type(1",
            ParseHtmlError::with_msg(
                "error while trying to read CSS refiner number after nth-last-of-type because end of string '(1' encountered before ending ')' was found",
            )),
            ("something-not-a-refiner",
             ParseHtmlError::with_msg("unknown refiner type something-not-a-refiner.")
            ),
        ];

        for t in tests {
            assert_eq!(
                parse_css_refiner(&mut t.0.chars().peekable()).unwrap_err(),
                t.1
            );
        }
    }

    #[test]
    fn parse_css_selector_relationship_test() {
        let tests = vec![
            (" > ", Ok(CssSelectorRelationshipType::Parent)),
            (">", Ok(CssSelectorRelationshipType::Parent)),
            ("   ", Ok(CssSelectorRelationshipType::Ancestor)),
            ("  ", Ok(CssSelectorRelationshipType::Ancestor)),
            (" ", Ok(CssSelectorRelationshipType::Ancestor)),
            (" ~ ", Ok(CssSelectorRelationshipType::PreviousSibling)),
            ("~", Ok(CssSelectorRelationshipType::PreviousSibling)),
            (" + ", Ok(CssSelectorRelationshipType::PreviousSiblingOnce)),
            ("+", Ok(CssSelectorRelationshipType::PreviousSiblingOnce)),
            (" > ~ ", Err(ParseHtmlError::with_msg("found multiple relationship seperators in selector first Parent and now PreviousSibling"))),
            ("~>", Err(ParseHtmlError::with_msg("found multiple relationship seperators in selector first PreviousSibling and now Parent"))),
        ];

        for t in tests {
            assert_eq!(
                parse_css_selector_relationship(&mut t.0.chars().peekable()),
                t.1
            );
        }
    }

    #[test]
    fn parse_css_selector_item_test() {
        let tests = vec![
            (
                "div",
                Ok(Some(CssSelectorItem {
                    tag: Some("div".to_owned()),
                    classes: None,
                    ids: None,
                    refiners: None,
                    attributes: None,
                })),
            ),
            (
                "div.c1",
                Ok(Some(CssSelectorItem {
                    tag: Some("div".to_owned()),
                    classes: Some(vec!["c1".to_owned()]),
                    ids: None,
                    refiners: None,
                    attributes: None,
                })),
            ),
            (
                "div.c1.c2",
                Ok(Some(CssSelectorItem {
                    tag: Some("div".to_owned()),
                    classes: Some(vec!["c1".to_owned(), "c2".to_owned()]),
                    ids: None,
                    refiners: None,
                    attributes: None,
                })),
            ),
            (
                "div#first",
                Ok(Some(CssSelectorItem {
                    tag: Some("div".to_owned()),
                    classes: None,
                    ids: Some(vec!["first".to_owned()]),
                    refiners: None,
                    attributes: None,
                })),
            ),
            (
                "div#first#second",
                Ok(Some(CssSelectorItem {
                    tag: Some("div".to_owned()),
                    classes: None,
                    ids: Some(vec!["first".to_owned(), "second".to_owned()]),
                    refiners: None,
                    attributes: None,
                })),
            ),
            (
                "div[attr]",
                Ok(Some(CssSelectorItem {
                    tag: Some("div".to_owned()),
                    classes: None,
                    ids: None,
                    refiners: None,
                    attributes: Some(vec![CssAttributeCompareType::Present("attr".to_owned())]),
                })),
            ),
            (
                "div:first-child[attr]",
                Ok(Some(CssSelectorItem {
                    tag: Some("div".to_owned()),
                    classes: None,
                    ids: None,
                    refiners: Some(vec![CssRefiner::FirstChild]),
                    attributes: Some(vec![CssAttributeCompareType::Present("attr".to_owned())]),
                })),
            ),
            (
                "div.c1:first-child[attr][attr2=1].c2#first:nth-of-type(2n+1)#second",
                Ok(Some(CssSelectorItem {
                    tag: Some("div".to_owned()),
                    classes: Some(vec!["c1".to_owned(), "c2".to_owned()]),
                    ids: Some(vec!["first".to_owned(), "second".to_owned()]),
                    refiners: Some(vec![
                        CssRefiner::FirstChild,
                        CssRefiner::NthOfType(CssRefinerNumberType::Functional((2, 1))),
                    ]),
                    attributes: Some(vec![
                        CssAttributeCompareType::Present("attr".to_owned()),
                        CssAttributeCompareType::Equals(("attr2".to_owned(), "1".to_owned())),
                    ]),
                })),
            ),
        ];

        for t in tests {
            assert_eq!(parse_css_selector_item(&mut t.0.chars().peekable()), t.1);
        }
    }

    #[test]
    fn parse_css_selector_rule_test() {
        let tests = vec![
            (
                "div",
                CssSelectorRule {
                    rules: vec![CssSelectorRelationship::Current(CssSelectorItem {
                        tag: Some("div".to_owned()),
                        classes: None,
                        ids: None,
                        refiners: None,
                        attributes: None,
                    })],
                },
            ),
            (
                "div p",
                CssSelectorRule {
                    rules: vec![
                        CssSelectorRelationship::Ancestor(CssSelectorItem {
                            tag: Some("div".to_owned()),
                            classes: None,
                            ids: None,
                            refiners: None,
                            attributes: None,
                        }),
                        CssSelectorRelationship::Current(CssSelectorItem {
                            tag: Some("p".to_owned()),
                            classes: None,
                            ids: None,
                            refiners: None,
                            attributes: None,
                        }),
                    ],
                },
            ),
            (
                "div > p",
                CssSelectorRule {
                    rules: vec![
                        CssSelectorRelationship::Parent(CssSelectorItem {
                            tag: Some("div".to_owned()),
                            classes: None,
                            ids: None,
                            refiners: None,
                            attributes: None,
                        }),
                        CssSelectorRelationship::Current(CssSelectorItem {
                            tag: Some("p".to_owned()),
                            classes: None,
                            ids: None,
                            refiners: None,
                            attributes: None,
                        }),
                    ],
                },
            ),
            (
                "div a > p",
                CssSelectorRule {
                    rules: vec![
                        CssSelectorRelationship::Ancestor(CssSelectorItem {
                            tag: Some("div".to_owned()),
                            classes: None,
                            ids: None,
                            refiners: None,
                            attributes: None,
                        }),
                        CssSelectorRelationship::Parent(CssSelectorItem {
                            tag: Some("a".to_owned()),
                            classes: None,
                            ids: None,
                            refiners: None,
                            attributes: None,
                        }),
                        CssSelectorRelationship::Current(CssSelectorItem {
                            tag: Some("p".to_owned()),
                            classes: None,
                            ids: None,
                            refiners: None,
                            attributes: None,
                        }),
                    ],
                },
            ),
            (
                "div a ~ p",
                CssSelectorRule {
                    rules: vec![
                        CssSelectorRelationship::Ancestor(CssSelectorItem {
                            tag: Some("div".to_owned()),
                            classes: None,
                            ids: None,
                            refiners: None,
                            attributes: None,
                        }),
                        CssSelectorRelationship::PreviousSibling(CssSelectorItem {
                            tag: Some("a".to_owned()),
                            classes: None,
                            ids: None,
                            refiners: None,
                            attributes: None,
                        }),
                        CssSelectorRelationship::Current(CssSelectorItem {
                            tag: Some("p".to_owned()),
                            classes: None,
                            ids: None,
                            refiners: None,
                            attributes: None,
                        }),
                    ],
                },
            ),
        ];

        for t in tests {
            assert_eq!(parse_css_selector_rule(t.0).unwrap(), t.1);
        }
    }
}

// *IMPROVEMENT IDEAS*

// 1.
// Standardise parsing functions -> create a parsing enum for types of checks
// EOF,
// Char(char),
// Substring(string),
//
// Then have a parse_until and parse_until_one_of functions.
// These functions should take a peekable, not a standard chars iterator

// 2.
// Make error messages more consistent in structure.
