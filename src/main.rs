use std::collections::HashMap;
use std::str::FromStr;

struct ParseHtmlError ;

impl ParseHtmlError {
    fn new() -> ParseHtmlError {
        ParseHtmlError
    }
}

impl std::fmt::Display for ParseHtmlError {
    fn fmt(&self, f : &mut  std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        write!(f, "Error occured parsing HTML");
        Ok(())
    }
}
impl std::fmt::Debug for ParseHtmlError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        write!(f, "Error occured parsing HTML");
        Ok(())
    }
}

struct HtmlNode {
    contents : Option<Vec<HtmlContent>>,
    class : Option<Vec<String>>,
    id : Option<Vec<String>>,
    attributes : Option<HashMap<String, String>>,
}

enum HtmlContent {
    Text(String),
    Node(HtmlNode),
}

struct CSSSelectorData <'a> {
    class : Option<&'a str>,
    id : Option<&'a str>,
}
enum CSSSelector <'a> {
    All,
    Specific(CSSSelectorData<'a>),
}

impl HtmlNode {
    fn new() -> HtmlNode {
        HtmlNode {class : None, id : None, contents : None}
    }

    fn Select(&self, selector : &CSSSelector) -> Option<Vec<&HtmlNode>> {
        let mut out : Vec<&HtmlNode> = Vec::new();

        match selector {
            CSSSelector::All => {
                out.push(self);
                ()
            },
            CSSSelector::Specific(selector_data) =>{
                let class_matches = match selector_data.class {
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
                let id_matches = match &selector_data.id {
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
                        }
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

enum ParseHtmlMode {
    InStartTag,
    InEndTag,
    InContent,
}

impl FromStr for HtmlNode {
    type Err = ParseHtmlError;
    fn from_str(html_str: &str) -> std::result::Result<Self, <Self as std::str::FromStr>::Err> {
        let mut parse_mode = ParseHtmlMode::InContent;
        let mut cur_slice : &str;
        let mut node = HtmlNode::new();
         for ch in html_str.chars() {
            if ch == '<' {
                match parse_mode {
                    ParseHtmlMode::InStartTag => return Err(ParseHtmlError::new()),
                    ParseHtmlMode::InEndTag => return Err(ParseHtmlError::new()),
                    ParseHtmlMode::InContent => {
                        //Need to work out how to determine it should be start or end tag...
                        //TODO: Follow idea in notebook - ParseUntil....
                        parse_mode = ParseHtmlMode::InStartTag;
                    }
                }
            }
         }
         Err(ParseHtmlError::new())
    }
}


fn main() {
    println!("Hello, world!");
}
