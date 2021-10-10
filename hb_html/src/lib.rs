mod error;
use error::HtmlDocError;

#[derive(Debug, PartialEq, Clone)]
struct Node_Id {
    index: usize,
}

impl Node_Id {
    fn new(index: usize) -> Node_Id {
        Node_Id { index: index }
    }
}

struct Html_Tag {
    parent: Option<Node_Id>,
    children: Option<Vec<Node_Id>>,
    //Html attributes
    tag: String,
    classes: Option<Vec<String>>,
    ids: Option<Vec<String>>,
}

struct Html_Text {
    parent: Option<Node_Id>,
    text: String,
}

enum Html_Data {
    Text(Html_Text),
    Tag(Html_Tag),
}

struct Html_Doc {
    ///Contains the information for the html document.
    ///Uses a memory arena to create a graph view.
    nodes: Vec<Html_Data>,
    top: Option<Node_Id>,
}

impl Html_Doc {
    fn new() -> Html_Doc {
        Html_Doc {
            nodes: vec![],
            top: None,
        }
    }

    fn add_top_node(&self, node: Html_Data) -> Result<Node_Id, HtmlDocError> {
        match self.top {
            Some(_) => Err(HtmlDocError::with_msg("Top level node already exists.")),
            None => {
                let id = Node_Id::new(self.nodes.len());
                self.nodes.push(node);
                self.top = Some(id.clone());
                Ok(id)
            }
        }
    }

    fn add_child_node(&self, node: Html_Data, parent: Node_Id) -> Result<Node_Id, HtmlDocError> {
        match self.get_node(&parent) {
            Err(_) => Err(HtmlDocError::with_msg("Invalid parent id.")),
            Ok(parent_node) => {
                match parent_node.node {
                    Html_Data::Text(_) => {
                        Err(HtmlDocError::with_msg("Cannont add child to text node."))
                    }
                    Html_Data::Tag(tag) => {
                        //make sure node has parent set
                        match &mut node {
                            Html_Data::Text(t) => match &mut t.parent {
                                None => t.parent = Some(parent.clone()),
                                Some(other) => {
                                    if other.index != parent.index {
                                        t.parent = Some(parent.clone());
                                    }
                                }
                            },
                        }
                        let id = Node_Id::new(self.nodes.len());
                        //add child to parent's children vec
                        match &mut tag.children {
                            None => tag.children = Some(vec![id.clone()]),
                            Some(vec) => vec.push(id.clone()),
                        }
                        self.nodes.push(node);
                        Ok(id)
                    }
                }
            }
        }
    }

    fn get_node(&self, id: &Node_Id) -> Result<Html_Node_Pointer, HtmlDocError> {
        let index = id.index;
        if index >= self.nodes.len() {
            return Err(HtmlDocError::with_msg("Invalid id."));
        }
        Ok(Html_Node_Pointer {
            doc: self,
            node: &self.nodes[index],
        })
    }
}

struct Html_Node_Pointer<'a> {
    ///This is a struct used for navigating the graph structure in the Html_Doc.
    doc: &'a Html_Doc,
    node: &'a Html_Data,
}

impl<'a> Html_Node_Pointer<'a> {
    fn new(doc: &'a Html_Doc, node: &'a Html_Data) -> Html_Node_Pointer<'a> {
        Html_Node_Pointer {
            doc: doc,
            node: node,
        }
    }

    fn Parent(&self) -> Option<Html_Node_Pointer> {
        match self.node {
            Html_Data::Text(t) => match &t.parent {
                None => None,
                Some(id) => match self.doc.get_node(id) {
                    Ok(n) => Some(n),
                    Err(_) => None,
                },
            },
            Html_Data::Tag(t) => match &t.parent {
                None => None,
                Some(id) => match self.doc.get_node(id) {
                    Ok(n) => Some(n),
                    Err(_) => None,
                },
            },
        }
    }

    fn Children(&self) -> Option<Vec<Html_Node_Pointer>> {
        match self.node {
            Html_Data::Text(_) => None,
            Html_Data::Tag(t) => match &t.children {
                None => None,
                Some(v) => {
                    let mut v_out = vec![];
                    for n in v {
                        match self.doc.get_node(n) {
                            Ok(result) => v_out.push(result),
                            Err(_) => (),
                        };
                    }
                    Some(v_out)
                }
            },
        }
    }
}

#[cfg(test)]
mod Html_Node_Pointer_Tests {
    use super::*;
    #[test]
    fn CheckParentRetreival() {
        let mut doc = Html_Doc::new();
    }
}
