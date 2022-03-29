use crate::objects::{CssRefiner, CssSelector, CssSelectorRelationship, HtmlNode};

pub trait HtmlQueryable {
    fn query(&self) -> HtmlQuery;
}

/// An object which points to the a node in the HTML tree including the path to
/// the node to allow looking at parent nodes.
pub struct HtmlQueryResult<'a> {
    /// The path down the tree.
    /// The node is found by dereferencing the last element of the vector
    pub path: Vec<(&'a Vec<HtmlNode>, usize)>,
}

impl<'a> HtmlQueryResult<'a> {
    /// Attempts to get the node pointed to by the path.
    /// Returns None if the path is empty.
    pub fn get_node(&self) -> Option<&HtmlNode> {
        if self.path.len() == 0 {
            return None;
        }
        let path_point = &self.path[self.path.len() - 1];
        return Some(&path_point.0[path_point.1]);
    }

    /// Attempts to get the parent to the node pointed to by the path.
    /// Returns None if the path is empty or is only the single node on the path.
    pub fn get_parent_node(&self) -> Option<&HtmlNode> {
        if self.path.len() <= 1 {
            return None;
        }
        let path_point = &self.path[self.path.len() - 2];
        return Some(&path_point.0[path_point.1]);
    }

    /// Attempts to get the node from the index position on the path.
    /// Returns None if the index is out of range of the path.
    pub fn get_node_from_index(&self, index: usize) -> Option<&HtmlNode> {
        if index >= self.path.len() {
            return None;
        }
        let path_point = &self.path[index];
        return Some(&path_point.0[path_point.1]);
    }

    /// Creates an iterator that walks the path from the bottom to the top.
    pub fn get_path_iter(&self) -> HtmlQueryResultIter {
        HtmlQueryResultIter::new(self)
    }

    /// Checks if the node pointed to matches the CSS style selector provided.
    pub fn matches<T: Into<CssSelector>>(&self, selector: T) -> bool {
        let selector = selector.into();
        match selector {
            CssSelector::Any => true,
            CssSelector::Specific(v) => {
                let mut passed = false;
                for selector_rule in v {
                    for rule in selector_rule.rules {
                        match rule {
                            CssSelectorRelationship::Current(selector_item) => {
                                // make sure it is a Html tag node
                                let tag_node = match self.get_node() {
                                    None => {
                                        break;
                                    }
                                    Some(n) => match n {
                                        HtmlNode::Tag(t) => t,
                                        // Not a tag node, don't care what it is otherwise
                                        _ => {
                                            break;
                                        }
                                    },
                                };
                                //Compare the tag selector
                                match selector_item.tag {
                                    None => (),
                                    Some(tag) => {
                                        if tag != tag_node.tag {
                                            //failed to match the tag, this selector rule failed
                                            break;
                                        }
                                    }
                                }

                                // check selector's classes
                                match selector_item.classes {
                                    None => (),
                                    Some(classes) => {
                                        let mut all_found = true;
                                        for class in classes {
                                            let mut found = false;
                                            for tag_class in tag_node.classes {
                                                if tag_class == class {
                                                    found = true;
                                                    break;
                                                }
                                            }
                                            //could not find one of the classes
                                            if found != true {
                                                all_found = false;
                                                break;
                                            }
                                        }
                                        //failed to find the classes, this selector rule failed
                                        if all_found != true {
                                            break;
                                        }
                                    }
                                }

                                // check selector's ids
                                match selector_item.ids {
                                    None => (),
                                    Some(ids) => {
                                        let mut all_found = true;
                                        for id in ids {
                                            let mut found = false;
                                            for tag_id in tag_node.ids {
                                                if tag_id == id {
                                                    found = true;
                                                    break;
                                                }
                                            }
                                            //could not find one of the ids
                                            if found != true {
                                                all_found = false;
                                                break;
                                            }
                                        }
                                        //failed to find the ids, this selector rule failed
                                        if all_found != true {
                                            break;
                                        }
                                    }
                                }

                                //check selector's refiners
                                match selector_item.refiners {
                                    None => (),
                                    Some(refiners) => {
                                        let mut all_found = true;
                                        for refiner in refiners {
                                            match refiner {
                                                CssRefiner::Checked => {
                                                    if tag_node.tag == "option".to_owned() {
                                                        if !tag_node
                                                            .attributes
                                                            .contains_key(&"selected".to_owned())
                                                        {
                                                            all_found = false;
                                                            break;
                                                        }
                                                    } else if tag_node.tag == "input".to_owned() {
                                                        //check type
                                                        let type_str = "type".to_owned();
                                                        if !tag_node
                                                            .attributes
                                                            .contains_key(&type_str)
                                                        {
                                                            all_found = false;
                                                            break;
                                                        }
                                                        if tag_node.attributes[&type_str]
                                                            != "checkbox".to_owned()
                                                            || tag_node.attributes[&type_str]
                                                                != "radio".to_owned()
                                                        {
                                                            all_found = false;
                                                            break;
                                                        }
                                                        //check if it is conatains the checked attribute - don't care about the value as it can be many different things
                                                        if !tag_node
                                                            .attributes
                                                            .contains_key(&"checked".to_owned())
                                                        {
                                                            all_found = false;
                                                            break;
                                                        }
                                                    } else {
                                                        all_found = false;
                                                        break;
                                                    }
                                                }
                                                CssRefiner::Default => {
                                                    // same as checked because this html parser does not have changing states
                                                    if tag_node.tag == "option".to_owned() {
                                                        if !tag_node
                                                            .attributes
                                                            .contains_key(&"selected".to_owned())
                                                        {
                                                            all_found = false;
                                                            break;
                                                        }
                                                    } else if tag_node.tag == "input".to_owned() {
                                                        //check type
                                                        let type_str = "type".to_owned();
                                                        if !tag_node
                                                            .attributes
                                                            .contains_key(&type_str)
                                                        {
                                                            all_found = false;
                                                            break;
                                                        }
                                                        if tag_node.attributes[&type_str]
                                                            != "checkbox".to_owned()
                                                            || tag_node.attributes[&type_str]
                                                                != "radio".to_owned()
                                                        {
                                                            all_found = false;
                                                            break;
                                                        }
                                                        //check if it is conatains the checked attribute - don't care about the value as it can be many different things
                                                        if !tag_node
                                                            .attributes
                                                            .contains_key(&"checked".to_owned())
                                                        {
                                                            all_found = false;
                                                            break;
                                                        }
                                                    } else {
                                                        all_found = false;
                                                        break;
                                                    }
                                                }
                                                CssRefiner::Disabled => {
                                                    // disabled attribute present on these tags
                                                    if tag_node.tag != "option".to_owned()
                                                        && tag_node.tag != "input".to_owned()
                                                        && tag_node.tag != "select".to_owned()
                                                        && tag_node.tag != "button".to_owned()
                                                        && tag_node.tag != "fieldset".to_owned()
                                                        && tag_node.tag != "optgroup".to_owned()
                                                        && tag_node.tag != "textarea".to_owned()
                                                    {
                                                        all_found = false;
                                                        break;
                                                    }
                                                    if !tag_node
                                                        .attributes
                                                        .contains_key(&"disabled".to_owned())
                                                    {
                                                        all_found = false;
                                                        break;
                                                    }
                                                }
                                                CssRefiner::Enabled => {
                                                    // disabled attribute not present on these tags
                                                    if tag_node.tag != "option".to_owned()
                                                        && tag_node.tag != "input".to_owned()
                                                        && tag_node.tag != "select".to_owned()
                                                        && tag_node.tag != "button".to_owned()
                                                        && tag_node.tag != "fieldset".to_owned()
                                                        && tag_node.tag != "optgroup".to_owned()
                                                        && tag_node.tag != "textarea".to_owned()
                                                    {
                                                        all_found = false;
                                                        break;
                                                    }
                                                    if tag_node
                                                        .attributes
                                                        .contains_key(&"disabled".to_owned())
                                                    {
                                                        all_found = false;
                                                        break;
                                                    }
                                                }
                                                CssRefiner::Invalid => (),
                                                CssRefiner::Valid => (),
                                                CssRefiner::Optional => (),
                                                CssRefiner::Required => (),
                                                CssRefiner::OutOfRange => (),
                                                CssRefiner::ReadOnly => (),
                                                CssRefiner::ReadWrite => (),
                                                CssRefiner::Empty => (),
                                                CssRefiner::FirstChild => (),
                                                CssRefiner::LastChild => (),
                                                CssRefiner::NthChild(num) => (),
                                                CssRefiner::NthLastChild(num) => (),
                                                CssRefiner::OnlyChild => (),
                                                CssRefiner::FirstOfType => (),
                                                CssRefiner::LastOfType => (),
                                                CssRefiner::NthOfType(num) => (),
                                                CssRefiner::NthLastOfType(num) => (),
                                                CssRefiner::OnlyOfType => (),
                                                CssRefiner::Not(not_selector) => {
                                                    if self.matches(not_selector) {
                                                        all_found = false;
                                                        break;
                                                    }
                                                }
                                                CssRefiner::Root => (),
                                            }
                                        }
                                    }
                                }
                            }
                            CssSelectorRelationship::Parent(selector_item) => (),
                            CssSelectorRelationship::Ancestor(selector_item) => (),
                            CssSelectorRelationship::PreviousSibling(selector_item) => (),
                            CssSelectorRelationship::PreviousSiblingOnce(selector_item) => (),
                        }
                    }
                }
                passed
            }
        }
    }
}

/// Iterator that walks along the path of the HtmlQueryResult from the bottom to
/// the top.
pub struct HtmlQueryResultIter<'a> {
    query_result: &'a HtmlQueryResult<'a>,
    previous_index: usize,
}
impl<'a> HtmlQueryResultIter<'a> {
    /// Create a HtmlQueryResultIter from a HtmlQueryResult reference.
    pub fn new(query_result: &'a HtmlQueryResult) -> HtmlQueryResultIter<'a> {
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
/// subsequent search will search from the existing results rather that
/// the top level of the HTML Document.
///
/// # Example
///
/// ```
/// use hb_html::objects::HtmlDocument;
/// use hb_html::querying::HtmlQuery;
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
/// let query = HtmlQuery::new(&html_doc.nodes);
/// query.find_with_tag("div").find_with_tag("p");
/// ```
/// This will find the div tags, then find the p tags from within the div tags.
pub struct HtmlQuery<'a> {
    pub root: &'a Vec<HtmlNode>,
    pub results: Vec<HtmlQueryResult<'a>>,
}

impl<'a> HtmlQuery<'a> {
    /// Creates a new HtmlQuery to search from the root nodes down.
    ///
    /// # Arguments
    ///
    /// * `root` - A reference to the vector of nodes that the Query object
    ///            begins searching from.
    pub fn new(root: &'a Vec<HtmlNode>) -> HtmlQuery<'a> {
        HtmlQuery {
            root: root,
            results: vec![],
        }
    }

    /// Find all elements with the tag provided in the Html structure.
    pub fn find_with_tag(&self, tag: &str) -> &HtmlQuery {
        todo!();
        // if there are existing results -> search using the results as the top level.
        //    loop through the results and call match();
        // else
        //     walk html tree calling match() on each node;
    }

    /// Search through either the root HTML nodes if there are no results stored,
    /// otherwise search through the current results.
    pub fn find(&self, selector: CssSelector) -> &HtmlQuery {
        todo!();
    }
}
pub struct HtmlQueryResultMut<'a> {
    pub path: Vec<(&'a mut Vec<HtmlNode>, usize)>,
}

pub struct HtmlQueryMut<'a> {
    pub root: &'a mut Vec<HtmlNode>,
    pub results: Vec<HtmlQueryResultMut<'a>>,
}

impl<'a> HtmlQueryMut<'a> {
    pub fn new(root: &'a mut Vec<HtmlNode>) -> HtmlQueryMut<'a> {
        HtmlQueryMut {
            root: root,
            results: vec![],
        }
    }
}
