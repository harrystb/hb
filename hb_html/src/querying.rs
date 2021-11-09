use crate::objects::HtmlNode;

pub trait HtmlQueryable {
    fn query(&self) -> HtmlQuery;
}

/// Provides a matches function that is used to check if a the objects matches the CSS Selector.
pub trait CSSMatchable {
    /// Check if the object matches the CSS selector provided.
    fn matches(&self, selector: &str) -> bool;
}

/// An object which points to the a node in the HTML tree including the path to
/// the node to allow looking at parent nodes.
pub struct HtmlQueryResult<'a> {
    /// The path down the tree.
    /// The node is found by dereferencing the last element of the vector
    pub path: Vec<(&'a Vec<HtmlNode>, usize)>,
}

impl CSSMatchable for HtmlQueryResult<'_> {
    fn matches(&self, selector: &str) -> bool {
        match self.get_node() {
            Some(node) => node.matches(selector),
            None => false,
        }
    }
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
    pub fn find(&self, tag: &str) -> &HtmlQuery {
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
