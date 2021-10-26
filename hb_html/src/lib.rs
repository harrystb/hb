//! Crate to parse and extract data from HTML documents.

pub mod error;
use error::{HtmlDocError, HtmlMatchError, ParseHtmlError};
pub mod objects;
use objects::{HtmlDocument, HtmlNode, HtmlTag};
pub mod parsing;

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
/// subsequent search will search from the existing results.
/// For example:
/// ```ignore
/// let query = HtmlQuery::new(&html_doc.nodes);
/// query.find_with_tag("div").find_with_tag("p")
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

    ///
    pub fn find_with_tag(&self, tag: &str) -> &HtmlQuery {
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
