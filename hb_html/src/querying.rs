use crate::error::{HtmlMatchError, ParseHtmlError};
use crate::objects::{
    CssAttributeCompareType, CssRefiner, CssRefinerNumberType, CssSelector, CssSelectorItem,
    CssSelectorRelationship, CssSelectorRule, HtmlDocument, HtmlNode, HtmlTag,
};
use std::collections::HashMap;
use std::convert::{TryFrom, TryInto};
use std::str::FromStr;

pub trait HtmlQueryable {
    fn query(&self) -> HtmlQuery;
}

/// An object which points to the a node in the HTML tree including the path to
/// the node to allow looking at parent nodes.
#[derive(Clone, PartialEq, Debug)]
pub struct HtmlQueryResult<'a> {
    /// The path down the tree.
    /// The node is found by dereferencing the last element of the vector
    pub path: Vec<(&'a Vec<HtmlNode>, usize)>,
}

impl<'a> HtmlQueryResult<'a> {
    /// Attempts to get the node pointed to by the path.
    /// Returns None if the path is empty.
    pub fn get_node(&self) -> Option<&'a HtmlNode> {
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

    pub fn move_to_parent(&mut self) -> Option<()> {
        if self.path.len() <= 1 {
            return None;
        }
        self.path.pop();
        Some(())
    }

    pub fn move_to_previous_sibling(&mut self) -> Option<()> {
        if self.path.len() == 0 {
            return None;
        }
        if self.path[self.path.len() - 1].1 == 0 {
            return None;
        }
        let mut previous_tag_sibling = None;
        for i in (0..self.path[self.path.len() - 1].1).rev() {
            match self.path[self.path.len() - 1].0[i] {
                HtmlNode::Tag(_) => {
                    previous_tag_sibling = Some(i);
                    break;
                }
                _ => (),
            }
        }
        match previous_tag_sibling {
            None => return None,
            Some(i) => {
                let mut end = self.path.pop().unwrap();
                end.1 = i;
                self.path.push(end);
            }
        }
        Some(())
    }
    pub fn move_to_next_sibling(&mut self) -> Option<()> {
        if self.path.len() == 0 {
            return None;
        }
        let (v, i) = self.path[self.path.len() - 1];
        if i + 1 == v.len() {
            return None;
        }
        let mut next_tag_sibling = None;
        for i in (self.path[self.path.len() - 1].1 + 1)..self.path[self.path.len() - 1].0.len() {
            match self.path[self.path.len() - 1].0[i] {
                HtmlNode::Tag(_) => {
                    next_tag_sibling = Some(i);
                    break;
                }
                _ => (),
            }
        }
        match next_tag_sibling {
            None => return None,
            Some(i) => {
                let mut end = self.path.pop().unwrap();
                end.1 = i;
                self.path.push(end);
            }
        }
        Some(())
    }
    pub fn move_to_first_child(&mut self) -> Option<()> {
        if self.path.len() == 0 {
            return None;
        }
        let (v, i) = self.path[self.path.len() - 1];
        match &v[i] {
            HtmlNode::Tag(t) => {
                if t.contents.len() > 0 {
                    self.path.push((&t.contents, 0));
                    Some(())
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    pub fn walk_next(&mut self) -> Option<()> {
        //attempt to move to first child
        match self.move_to_first_child() {
            Some(a) => return Some(a),
            None => (),
        };
        // next try go to next sibling
        match self.move_to_next_sibling() {
            Some(a) => Some(a),
            None => loop {
                // now to go to parents next sibling, and keep going up until the end or a next sibling is found
                match self.move_to_parent() {
                    None => {
                        return None;
                    }
                    Some(_) => match self.move_to_next_sibling() {
                        Some(a) => return Some(a),
                        None => (),
                    },
                }
            },
        }
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

    fn matches_item(&self, selector_item: &CssSelectorItem) -> bool {
        // make sure it is a Html tag node
        let tag_node = match self.get_node() {
            None => {
                return false;
            }
            Some(n) => match n {
                HtmlNode::Tag(t) => t,
                // Not a tag node, don't care what it is otherwise
                _ => {
                    return false;
                }
            },
        };
        //Compare the tag selector
        match &selector_item.tag {
            None => (),
            Some(tag) => {
                if *tag != tag_node.tag {
                    //failed to match the tag, this selector rule failed
                    return false;
                }
            }
        }

        // check selector's classes
        match &selector_item.classes {
            None => (),
            Some(classes) => {
                let mut all_found = true;
                for class in classes {
                    let mut found = false;
                    for tag_class in &tag_node.classes {
                        if *tag_class == *class {
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
                    return false;
                }
            }
        }

        // check selector's ids
        match &selector_item.ids {
            None => (),
            Some(ids) => {
                let mut all_found = true;
                for id in ids {
                    let mut found = false;
                    for tag_id in &tag_node.ids {
                        if *tag_id == *id {
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
                    return false;
                }
            }
        }

        //check selector's refiners
        match &selector_item.refiners {
            None => (),
            Some(refiners) => {
                let mut all_found = true;
                for refiner in refiners {
                    match refiner {
                        CssRefiner::Checked => {
                            if tag_node.tag == "option".to_owned() {
                                if !tag_node.attributes.contains_key(&"selected".to_owned()) {
                                    all_found = false;
                                    break;
                                }
                                if tag_node.attributes[&"selected".to_owned()] == "false" {
                                    all_found = false;
                                    break;
                                }
                            } else if &tag_node.tag == "input" {
                                //check type
                                let type_str = "type".to_owned();
                                if !tag_node.attributes.contains_key(&type_str) {
                                    all_found = false;
                                    break;
                                }
                                if tag_node.attributes[&type_str] != "checkbox".to_owned()
                                    && tag_node.attributes[&type_str] != "radio".to_owned()
                                {
                                    all_found = false;
                                    break;
                                }
                                //check if it is conatains the checked attribute - don't care about the value as it can be many different things
                                if !tag_node.attributes.contains_key(&"checked".to_owned()) {
                                    all_found = false;
                                    break;
                                }
                                if tag_node.attributes[&"checked".to_owned()] == "false" {
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
                                if !tag_node.attributes.contains_key(&"selected".to_owned()) {
                                    all_found = false;
                                    break;
                                }
                                if tag_node.attributes[&"selected".to_owned()] == "false" {
                                    all_found = false;
                                    break;
                                }
                            } else if tag_node.tag == "input".to_owned() {
                                //check type
                                let type_str = "type".to_owned();
                                if !tag_node.attributes.contains_key(&type_str) {
                                    all_found = false;
                                    break;
                                }
                                if tag_node.attributes[&type_str] != "checkbox".to_owned()
                                    && tag_node.attributes[&type_str] != "radio".to_owned()
                                {
                                    all_found = false;
                                    break;
                                }
                                //check if it is conatains the checked attribute - don't care about the value as it can be many different things
                                if !tag_node.attributes.contains_key(&"checked".to_owned()) {
                                    all_found = false;
                                    break;
                                }
                                if tag_node.attributes[&"checked".to_owned()] == "false" {
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
                            if !tag_node.attributes.contains_key(&"disabled".to_owned()) {
                                all_found = false;
                                break;
                            }
                            if tag_node.attributes[&"disabled".to_owned()] == "false" {
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
                            if tag_node.attributes.contains_key(&"disabled".to_owned()) {
                                if tag_node.attributes[&"disabled".to_owned()] != "false" {
                                    all_found = false;
                                    break;
                                }
                            }
                        }
                        CssRefiner::Optional => {
                            // required attribute not present on these tags
                            if tag_node.tag != "input".to_owned()
                                && tag_node.tag != "select".to_owned()
                                && tag_node.tag != "textarea".to_owned()
                            {
                                all_found = false;
                                break;
                            }
                            if tag_node.attributes.contains_key(&"required".to_owned()) {
                                if tag_node.attributes[&"required".to_owned()] != "false" {
                                    all_found = false;
                                    break;
                                }
                            }
                        }
                        CssRefiner::Required => {
                            // required attribute present on these tags
                            if tag_node.tag != "input".to_owned()
                                && tag_node.tag != "select".to_owned()
                                && tag_node.tag != "textarea".to_owned()
                            {
                                all_found = false;
                                break;
                            }
                            if !tag_node.attributes.contains_key(&"required".to_owned()) {
                                all_found = false;
                                break;
                            }
                            if tag_node.attributes[&"required".to_owned()] == "false" {
                                all_found = false;
                                break;
                            }
                        }
                        CssRefiner::ReadOnly => {
                            // editable tags with read-only attribute or
                            // non-standard editable with contenteditable="" or "true"
                            if tag_node.tag == "input".to_owned()
                                || tag_node.tag == "textarea".to_owned()
                            {
                                if !tag_node.attributes.contains_key(&"read-only".to_owned()) {
                                    all_found = false;
                                    break;
                                }
                                if tag_node.attributes[&"read-only".to_owned()] == "false" {
                                    all_found = false;
                                    break;
                                }
                            } else {
                                if tag_node
                                    .attributes
                                    .contains_key(&"contenteditable".to_owned())
                                {
                                    if tag_node.attributes[&"contenteditable".to_owned()] != "false"
                                    {
                                        all_found = false;
                                        break;
                                    }
                                }
                            }
                        }
                        CssRefiner::ReadWrite => {
                            // editable tags with read-only attribute or
                            // non-standard editable with contenteditable="" or "true"
                            if tag_node.tag == "input".to_owned()
                                || tag_node.tag == "textarea".to_owned()
                            {
                                if tag_node.attributes.contains_key(&"read-only".to_owned()) {
                                    if tag_node.attributes[&"read-only".to_owned()] != "false" {
                                        all_found = false;
                                        break;
                                    }
                                }
                            } else {
                                if !tag_node
                                    .attributes
                                    .contains_key(&"contenteditable".to_owned())
                                {
                                    all_found = false;
                                    break;
                                }
                                if tag_node.attributes[&"contenteditable".to_owned()] == "false" {
                                    all_found = false;
                                    break;
                                }
                            }
                        }
                        CssRefiner::Empty => {
                            for content in &tag_node.contents {
                                match content {
                                    HtmlNode::Tag(_) => {
                                        all_found = false;
                                        break;
                                    }
                                    HtmlNode::Comment(_) => (),
                                    HtmlNode::Text(s) => {
                                        let mut found_non_whitespace = false;
                                        for c in s.chars() {
                                            if !c.is_ascii_whitespace() {
                                                found_non_whitespace = true;
                                                break;
                                            }
                                        }
                                        if found_non_whitespace {
                                            all_found = false;
                                            break;
                                        }
                                    }
                                }
                            }
                        }
                        CssRefiner::FirstChild => {
                            let path_point = match self.path.last() {
                                None => {
                                    all_found = false;
                                    break;
                                }
                                Some(p) => p,
                            };
                            if path_point.1 != 0 {
                                //not the first index, check if there are any tags before it
                                let mut found_extra_tag = false;
                                for (i, node) in path_point.0.iter().enumerate() {
                                    if i == path_point.1 {
                                        break;
                                    }
                                    match node {
                                        HtmlNode::Tag(_) => {
                                            found_extra_tag = true;
                                            break;
                                        }
                                        _ => (),
                                    }
                                }
                                if found_extra_tag {
                                    all_found = false;
                                    break;
                                }
                            }
                        }
                        CssRefiner::LastChild => {
                            let path_point = match self.path.last() {
                                None => {
                                    all_found = false;
                                    break;
                                }
                                Some(p) => p,
                            };
                            if path_point.1 != path_point.0.len() - 1 {
                                //not the last index, check if there are any tag nodes after it
                                let mut iter = path_point.0.iter();
                                iter.nth(path_point.1); //consume up to the pointed to object
                                let mut found_extra_tag = false;
                                for node in iter {
                                    match node {
                                        HtmlNode::Tag(_) => {
                                            found_extra_tag = true;
                                            break;
                                        }
                                        _ => (),
                                    }
                                }
                                if found_extra_tag {
                                    all_found = false;
                                    break;
                                }
                            }
                        }
                        CssRefiner::NthChild(num) => {
                            let path_point = match self.path.last() {
                                None => {
                                    all_found = false;
                                    break;
                                }
                                Some(p) => p,
                            };
                            let mut tag_count = 0;
                            let mut number_from_start = 0;
                            for (i, child) in path_point.0.iter().enumerate() {
                                match child {
                                    HtmlNode::Tag(_) => {
                                        tag_count += 1;
                                        if i == path_point.1 {
                                            number_from_start = tag_count;
                                            break;
                                        }
                                    }
                                    _ => (),
                                }
                            }
                            match num {
                                CssRefinerNumberType::Even => {
                                    if number_from_start % 2 != 0 {
                                        // index 0 == 1st
                                        all_found = false;
                                        break;
                                    }
                                }
                                CssRefinerNumberType::Odd => {
                                    if number_from_start % 2 != 1 {
                                        // index 0 == 1st
                                        all_found = false;
                                        break;
                                    }
                                }
                                CssRefinerNumberType::Specific(val) => {
                                    if number_from_start != *val {
                                        all_found = false;
                                        break;
                                    }
                                }
                                CssRefinerNumberType::Functional((step, offset)) => {
                                    let number_from_start_i32 =
                                        match i32::try_from(number_from_start) {
                                            // + 1 to convert to number from start
                                            Err(_) => {
                                                //Path too large to use with functional
                                                all_found = false;
                                                break;
                                            }
                                            Ok(a) => a,
                                        };
                                    if *step < 0 {
                                        if number_from_start_i32 - offset > 0 {
                                            all_found = false;
                                            break;
                                        }
                                        if (number_from_start_i32 - offset) % step != 0 {
                                            // An+B = x -> (x-B) = An for any n -> (x-B) % A == 0
                                            all_found = false;
                                            break;
                                        }
                                    } else if *step == 0 {
                                        if number_from_start_i32 != *offset {
                                            all_found = false;
                                            break;
                                        }
                                    } else {
                                        if number_from_start_i32 - offset < 0 {
                                            all_found = false;
                                            break;
                                        }
                                        if (number_from_start_i32 - offset) % step != 0 {
                                            // An+B = x -> (x-B) = An for any n -> (x-B) % A == 0
                                            all_found = false;
                                            break;
                                        }
                                    }
                                }
                            }
                        }
                        CssRefiner::NthLastChild(num) => {
                            let path_point = match self.path.last() {
                                None => {
                                    all_found = false;
                                    break;
                                }
                                Some(p) => p,
                            };
                            let pos = path_point.0.len() - path_point.1 - 1;
                            let mut tag_count = 0;
                            let mut number_from_end = 0;
                            for (i, child) in path_point.0.iter().rev().enumerate() {
                                match child {
                                    HtmlNode::Tag(_) => {
                                        tag_count += 1;
                                        if i == pos {
                                            number_from_end = tag_count;
                                            break;
                                        }
                                    }
                                    _ => (),
                                }
                            }
                            match num {
                                CssRefinerNumberType::Even => {
                                    if number_from_end % 2 != 0 {
                                        // index 0 == 1st
                                        all_found = false;
                                        break;
                                    }
                                }
                                CssRefinerNumberType::Odd => {
                                    if number_from_end % 2 != 1 {
                                        // index 0 == 1st
                                        all_found = false;
                                        break;
                                    }
                                }
                                CssRefinerNumberType::Specific(val) => {
                                    if number_from_end != *val {
                                        all_found = false;
                                        break;
                                    }
                                }
                                CssRefinerNumberType::Functional((step, offset)) => {
                                    let number_from_end_i32 = match i32::try_from(number_from_end) {
                                        Err(_) => {
                                            //Path too large to use with functional
                                            all_found = false;
                                            break;
                                        }
                                        Ok(a) => a,
                                    };
                                    if *step < 0 {
                                        if number_from_end_i32 - offset > 0 {
                                            all_found = false;
                                            break;
                                        }
                                        if (number_from_end_i32 - offset) % step != 0 {
                                            // An+B = x -> (x-B) = An for any n -> (x-B) % A == 0
                                            all_found = false;
                                            break;
                                        }
                                    } else if *step == 0 {
                                        if number_from_end_i32 != *offset {
                                            all_found = false;
                                            break;
                                        }
                                    } else {
                                        if number_from_end_i32 - offset < 0 {
                                            all_found = false;
                                            break;
                                        }
                                        if (number_from_end_i32 - offset) % step != 0 {
                                            // An+B = x -> (x-B) = An for any n -> (x-B) % A == 0
                                            all_found = false;
                                            break;
                                        }
                                    }
                                }
                            }
                        }
                        CssRefiner::OnlyChild => {
                            let path_point = match self.path.last() {
                                None => {
                                    all_found = false;
                                    break;
                                }
                                Some(p) => p,
                            };
                            let mut found_other = false;
                            for (i, child) in path_point.0.iter().enumerate() {
                                match child {
                                    HtmlNode::Tag(_) => {
                                        if i != path_point.1 {
                                            found_other = true;
                                            break;
                                        }
                                    }
                                    _ => (),
                                }
                            }
                            if found_other {
                                all_found = false;
                                break;
                            }
                        }
                        CssRefiner::FirstOfType => {
                            let path_point = match self.path.last() {
                                None => {
                                    all_found = false;
                                    break;
                                }
                                Some(p) => p,
                            };
                            let mut found_other = false;
                            for (i, child) in path_point.0.iter().enumerate() {
                                match child {
                                    HtmlNode::Tag(t) => {
                                        //first tag that matches
                                        if t.tag == tag_node.tag {
                                            // fail if it is not the one we are looking at
                                            if i != path_point.1 {
                                                found_other = true;
                                            }
                                            break;
                                        }
                                    }
                                    _ => (),
                                }
                            }
                            if found_other {
                                all_found = false;
                                break;
                            }
                        }
                        CssRefiner::LastOfType => {
                            let path_point = match self.path.last() {
                                None => {
                                    all_found = false;
                                    break;
                                }
                                Some(p) => p,
                            };
                            let pos = path_point.0.len() - path_point.1 - 1;
                            let mut found_other = false;
                            for (i, child) in path_point.0.iter().rev().enumerate() {
                                match child {
                                    HtmlNode::Tag(t) => {
                                        //first tag that matches
                                        if t.tag == tag_node.tag {
                                            // fail if it is not the one we are looking at
                                            if i != pos {
                                                found_other = true;
                                            }
                                            break;
                                        }
                                    }
                                    _ => (),
                                }
                            }
                            if found_other {
                                all_found = false;
                                break;
                            }
                        }
                        CssRefiner::NthOfType(num) => {
                            let path_point = match self.path.last() {
                                None => {
                                    all_found = false;
                                    break;
                                }
                                Some(p) => p,
                            };
                            let mut tag_count = 0;
                            let mut number_from_start = 0;
                            for (i, child) in path_point.0.iter().enumerate() {
                                match child {
                                    HtmlNode::Tag(t) => {
                                        if t.tag == tag_node.tag {
                                            tag_count += 1;
                                        }
                                        if i == path_point.1 {
                                            number_from_start = tag_count;
                                            break;
                                        }
                                    }
                                    _ => (),
                                }
                            }
                            match num {
                                CssRefinerNumberType::Even => {
                                    if number_from_start % 2 != 0 {
                                        // index 0 == 1st
                                        all_found = false;
                                        break;
                                    }
                                }
                                CssRefinerNumberType::Odd => {
                                    if number_from_start % 2 != 1 {
                                        // index 0 == 1st
                                        all_found = false;
                                        break;
                                    }
                                }
                                CssRefinerNumberType::Specific(val) => {
                                    if number_from_start != *val {
                                        all_found = false;
                                        break;
                                    }
                                }
                                CssRefinerNumberType::Functional((step, offset)) => {
                                    let number_from_end_i32 = match i32::try_from(number_from_start)
                                    {
                                        Err(_) => {
                                            //Path too large to use with functional
                                            all_found = false;
                                            break;
                                        }
                                        Ok(a) => a,
                                    };
                                    if *step < 0 {
                                        if number_from_end_i32 - offset > 0 {
                                            all_found = false;
                                            break;
                                        }
                                        if (number_from_end_i32 - offset) % step != 0 {
                                            // An+B = x -> (x-B) = An for any n -> (x-B) % A == 0
                                            all_found = false;
                                            break;
                                        }
                                    } else if *step == 0 {
                                        if number_from_end_i32 != *offset {
                                            all_found = false;
                                            break;
                                        }
                                    } else {
                                        if number_from_end_i32 - offset < 0 {
                                            all_found = false;
                                            break;
                                        }
                                        if (number_from_end_i32 - offset) % step != 0 {
                                            // An+B = x -> (x-B) = An for any n -> (x-B) % A == 0
                                            all_found = false;
                                            break;
                                        }
                                    }
                                }
                            }
                        }
                        CssRefiner::NthLastOfType(num) => {
                            let path_point = match self.path.last() {
                                None => {
                                    all_found = false;
                                    break;
                                }
                                Some(p) => p,
                            };
                            let pos = path_point.0.len() - path_point.1 - 1;
                            let mut tag_count = 0;
                            let mut number_from_end = 0;
                            for (i, child) in path_point.0.iter().rev().enumerate() {
                                match child {
                                    HtmlNode::Tag(t) => {
                                        if t.tag == tag_node.tag {
                                            tag_count += 1;
                                        }
                                        if i == pos {
                                            number_from_end = tag_count;
                                            break;
                                        }
                                    }
                                    _ => (),
                                }
                            }
                            match num {
                                CssRefinerNumberType::Even => {
                                    if number_from_end % 2 != 0 {
                                        // index 0 == 1st
                                        all_found = false;
                                        break;
                                    }
                                }
                                CssRefinerNumberType::Odd => {
                                    if number_from_end % 2 != 1 {
                                        // index 0 == 1st
                                        all_found = false;
                                        break;
                                    }
                                }
                                CssRefinerNumberType::Specific(val) => {
                                    if number_from_end != *val {
                                        all_found = false;
                                        break;
                                    }
                                }
                                CssRefinerNumberType::Functional((step, offset)) => {
                                    let number_from_end_i32 = match i32::try_from(number_from_end) {
                                        Err(_) => {
                                            //Path too large to use with functional
                                            all_found = false;
                                            break;
                                        }
                                        Ok(a) => a,
                                    };
                                    if *step < 0 {
                                        if number_from_end_i32 - offset > 0 {
                                            all_found = false;
                                            break;
                                        }
                                        if (number_from_end_i32 - offset) % step != 0 {
                                            // An+B = x -> (x-B) = An for any n -> (x-B) % A == 0
                                            all_found = false;
                                            break;
                                        }
                                    } else if *step == 0 {
                                        if number_from_end_i32 != *offset {
                                            all_found = false;
                                            break;
                                        }
                                    } else {
                                        if number_from_end_i32 - offset < 0 {
                                            all_found = false;
                                            break;
                                        }
                                        if (number_from_end_i32 - offset) % step != 0 {
                                            // An+B = x -> (x-B) = An for any n -> (x-B) % A == 0
                                            all_found = false;
                                            break;
                                        }
                                    }
                                }
                            }
                        }
                        CssRefiner::OnlyOfType => {
                            let path_point = match self.path.last() {
                                None => {
                                    all_found = false;
                                    break;
                                }
                                Some(p) => p,
                            };
                            let mut found_other = false;
                            for (i, child) in path_point.0.iter().enumerate() {
                                match child {
                                    HtmlNode::Tag(t) => {
                                        if t.tag == tag_node.tag {
                                            if i != path_point.1 {
                                                found_other = true;
                                                break;
                                            }
                                        }
                                    }
                                    _ => (),
                                }
                            }
                            if found_other {
                                all_found = false;
                                break;
                            }
                        }
                        CssRefiner::Not(not_selector) => {
                            if self.matches(not_selector) {
                                all_found = false;
                                break;
                            }
                        }
                        CssRefiner::Root => {
                            if self.path.len() != 1 {
                                all_found = false;
                                break;
                            }
                        }
                    }
                }
                if all_found != true {
                    return false;
                }
            }
        }
        match &selector_item.attributes {
            None => (),
            Some(attr_comps) => {
                let mut all_found = true;
                for attr_comp in attr_comps {
                    match attr_comp {
                        CssAttributeCompareType::Present(a) => {
                            if !tag_node.attributes.contains_key(a) {
                                all_found = false;
                                break;
                            }
                        }
                        CssAttributeCompareType::Equals((a, val)) => {
                            if !tag_node.attributes.contains_key(a) {
                                all_found = false;
                                break;
                            }
                            if tag_node.attributes[a] != *val {
                                all_found = false;
                                break;
                            }
                        }
                        CssAttributeCompareType::Contains((a, val)) => {
                            if !tag_node.attributes.contains_key(a) {
                                all_found = false;
                                break;
                            }
                            if !tag_node.attributes[a].contains(val) {
                                all_found = false;
                                break;
                            }
                        }
                        CssAttributeCompareType::BeginsWith((a, val)) => {
                            if !tag_node.attributes.contains_key(a) {
                                all_found = false;
                                break;
                            }
                            if !tag_node.attributes[a].starts_with(val) {
                                all_found = false;
                                break;
                            }
                        }
                        CssAttributeCompareType::EqualsOrBeingsWith((a, val)) => {
                            if !tag_node.attributes.contains_key(a) {
                                all_found = false;
                                break;
                            }
                            let mut alt_val = val.to_owned();
                            alt_val.push('-');
                            if tag_node.attributes[a] != *val
                                && !tag_node.attributes[a].starts_with(&alt_val)
                            {
                                all_found = false;
                                break;
                            }
                        }
                        CssAttributeCompareType::EndsWith((a, val)) => {
                            if !tag_node.attributes.contains_key(a) {
                                all_found = false;
                                break;
                            }
                            if !tag_node.attributes[a].ends_with(val) {
                                all_found = false;
                                break;
                            }
                        }
                        CssAttributeCompareType::ContainsWord((a, val)) => {
                            if !tag_node.attributes.contains_key(a) {
                                all_found = false;
                                break;
                            }
                            let mut word_found = false;
                            for word in tag_node.attributes[a].split_whitespace() {
                                if word == val {
                                    word_found = true;
                                    break;
                                }
                            }
                            if !word_found {
                                all_found = false;
                                break;
                            }
                        }
                    }
                }
                if all_found != true {
                    return false;
                }
            }
        }
        true
    }

    fn matches_selector_rule(&self, selector_rule: &CssSelectorRule) -> bool {
        let mut moveable_pointer = self.clone();
        let mut mut_selector_rule = selector_rule.clone();
        while let Some(rule) = mut_selector_rule.rules.pop() {
            match rule {
                CssSelectorRelationship::Current(selector_item) => {
                    if !self.matches_item(&selector_item) {
                        return false;
                    }
                }
                CssSelectorRelationship::Parent(selector_item) => {
                    match moveable_pointer.move_to_parent() {
                        Some(_) => {
                            if !moveable_pointer.matches_item(&selector_item) {
                                return false;
                            }
                        }
                        None => {
                            return false;
                        }
                    }
                }
                CssSelectorRelationship::Ancestor(selector_item) => {
                    while let Some(_) = moveable_pointer.move_to_parent() {
                        if moveable_pointer.matches_item(&selector_item) {
                            match moveable_pointer.matches_selector_rule(&mut_selector_rule) {
                                true => return true, // We checked the rest of the rules using the moveable_pointer
                                false => (),
                            }
                        }
                    }
                    return false;
                }
                CssSelectorRelationship::PreviousSibling(selector_item) => {
                    let mut one_matches = false;
                    while let Some(_) = moveable_pointer.move_to_previous_sibling() {
                        if moveable_pointer.matches_item(&selector_item) {
                            one_matches = true;
                            break;
                        }
                    }
                    if !one_matches {
                        return false;
                    }
                }
                CssSelectorRelationship::PreviousSiblingOnce(selector_item) => {
                    match moveable_pointer.move_to_previous_sibling() {
                        None => {
                            return false;
                        }
                        Some(_) => {
                            if !moveable_pointer.matches_item(&selector_item) {
                                return false;
                            }
                        }
                    }
                }
            }
        }
        true
    }

    /// Checks if the node pointed to matches the CSS style selector provided.
    pub fn matches<T: Into<&'a CssSelector>>(&self, selector: T) -> bool {
        let selector = selector.into();
        match selector {
            CssSelector::Any => true,
            CssSelector::Specific(v) => {
                for selector_rule in v {
                    match self.matches_selector_rule(&selector_rule) {
                        true => return true,
                        false => (),
                    }
                }
                false
            }
        }
    }
}

#[cfg(test)]
mod html_match_tests {
    use super::*;

    #[test]
    fn html_matching_basic_test() {
        let doc = HtmlDocument::from_str(
            "<html><body><div>Hello <p class=bold>app</p></div></body></html>",
        )
        .unwrap();
        let mut q = doc.query();
        q.find_str("div");
        assert_eq!(
            q.results
                .iter()
                .map(|x| x.get_node().unwrap())
                .collect::<Vec<&HtmlNode>>(),
            vec![&HtmlNode::Tag(HtmlTag {
                tag: "div".to_owned(),
                ids: [].to_vec(),
                classes: [].to_vec(),
                attributes: HashMap::new(),
                contents: [
                    HtmlNode::Text("Hello ".to_owned()),
                    HtmlNode::Tag(HtmlTag {
                        tag: "p".to_owned(),
                        ids: [].to_vec(),
                        classes: ["bold".to_owned()].to_vec(),
                        attributes: HashMap::new(),
                        contents: [HtmlNode::Text("app".to_owned())].to_vec()
                    })
                ]
                .to_vec()
            })]
        );
        q.find_str("p").unwrap();
        assert_eq!(
            q.results
                .iter()
                .map(|x| x.get_node().unwrap())
                .collect::<Vec<&HtmlNode>>(),
            vec![&HtmlNode::Tag(HtmlTag {
                tag: "p".to_owned(),
                ids: [].to_vec(),
                classes: ["bold".to_owned()].to_vec(),
                attributes: HashMap::new(),
                contents: [HtmlNode::Text("app".to_owned())].to_vec()
            })]
        );
    }

    #[test]
    fn html_matching_specific_tests() {
        let doc = HtmlDocument::from_str(
            r#"<!DOCTYPE html>
<html>
    <head>
        <meta http-equiv="Content-Type" content="text/html; charset=UTF-8">
        <title>A Sample HTML Document</title>
        <meta name="description" content="A HTML document for testing purposes.">
        <meta name="author" content="HB">
        <meta name="viewport" content="width=device-width, initial-scale=1">
    </head>
    <body>
        <h1 class=heading>Test HTML Document</h1>
        <p>A HTML document for testing purposes.</p>
        <div class="listbox shadow">List of dairy items:
            <ul>
                <li>Milk</li>
                <li>Cheese</li>
                <li>Yoghurt</li>
                <li>Cream</li>
            </ul>
        </div>
        <div class=top><div class=alone></div>That div is empty...</div>
        <div class="tablebox shadow">Table of values:
            <table>
                <tr>
                    <th>Company</th>
                    <th>Contact</th>
                    <th>Country</th>
                </tr>
                <tr id=first-data-row>
                    <td>Alfreds Futterkiste</td>
                    <td>Maria Anders</td>
                    <td>Germany</td>
                </tr>
                <tr custom=red>
                    <td custom=reds>Centro comercial Moctezuma</td>
                    <td custom=re-ds>Francisco Chang</td>
                    <td custom="something else">Mexico</td>
                </tr>
            </table>
        </div>
        <div>Form stuffs
            <form>
              First name: <input type="text" value="Mickey"><br>
              Last name: <input type="text" value="Mouse"><br>
              Country: <input type="text" disabled="disabled" value="Disneyland">
              <input type="radio" checked="checked" value="male" name="gender" read-only=read-only> Male<br>
              <input type="radio" value="female" name="gender" required=required read-only=false> Female<br>
              <input type="checkbox" checked="checked" value="Bike" required> I have a bike<br>
              <input type="checkbox" value="Car" read-only> I have a car 
            </form>
        </div>
        <div> Options
            <select>
              <option value="blue" selected>Blue</option>
              <option value="red">Red</option>
              <option value="green">Green</option>
              <option value="yellow">Yellow</option>
            </select>
        </div>
    </body>
</html>"#,
        )
        .unwrap();
        let doc_manual = HtmlDocument {
            doctype: "html".to_owned(),
            nodes: vec![HtmlNode::Tag(HtmlTag::new("html").contents(vec![
                HtmlNode::new_text("\n    "),
                HtmlNode::Tag(HtmlTag::new("head").contents(vec![
                        HtmlNode::new_text("\n        "),
                        HtmlNode::Tag(HtmlTag::new("meta").attributes(vec![
                            ("content", "text/html; charset=UTF-8"),
                            ("http-equiv", "Content-Type"),
                        ])),
                        HtmlNode::new_text("\n        "),
                        HtmlNode::Tag(
                            HtmlTag::new("title").contents(vec![HtmlNode::Text(
                                "A Sample HTML Document".to_owned(),
                            )]),
                        ),
                        HtmlNode::new_text("\n        "),
                        HtmlNode::Tag(HtmlTag::new("meta").attributes(vec![
                            ("name", "description"),
                            ("content", "A HTML document for testing purposes."),
                        ])),
                        HtmlNode::new_text("\n        "),
                        HtmlNode::Tag(HtmlTag::new("meta").attributes(vec![
                            ("name".to_owned(), "author".to_owned()),
                            ("content".to_owned(), "HB".to_owned()),
                        ])),
                        HtmlNode::new_text("\n        "),
                        HtmlNode::Tag(HtmlTag::new("meta").attributes(vec![
                            ("name", "viewport"),
                            ("content", "width=device-width, initial-scale=1"),
                        ])),
                        HtmlNode::new_text("\n    "),
                    ])),
                HtmlNode::new_text("\n    "),
                HtmlNode::Tag(HtmlTag::new("body").contents(vec![
                            HtmlNode::new_text("\n        "),
                            HtmlNode::Tag(
                                HtmlTag::new("h1")
                                    .classes(vec!["heading"])
                                    .contents(vec![HtmlNode::new_text("Test HTML Document")]),
                            ),
                            HtmlNode::new_text("\n        "),
                            HtmlNode::Tag(HtmlTag::new("p").contents(vec![HtmlNode::new_text(
                                "A HTML document for testing purposes.",
                            )])),
                            HtmlNode::new_text("\n        "),
                            HtmlNode::Tag(
                                HtmlTag::new("div")
                                    .classes(vec!["listbox", "shadow"])
                                    .contents(vec![
                                        HtmlNode::new_text("List of dairy items:\n            "),
                                        HtmlNode::Tag(HtmlTag::new("ul").contents(vec![
                                                HtmlNode::new_text("\n                "),
                                                HtmlNode::Tag(
                                                    HtmlTag::new("li")
                                                        .contents(vec![HtmlNode::new_text("Milk")]),
                                                ),
                                                HtmlNode::new_text("\n                "),
                                                HtmlNode::Tag(
                                                    HtmlTag::new("li").contents(vec![
                                                        HtmlNode::new_text("Cheese"),
                                                    ]),
                                                ),
                                                HtmlNode::new_text("\n                "),
                                                HtmlNode::Tag(
                                                    HtmlTag::new("li").contents(vec![
                                                        HtmlNode::new_text("Yoghurt"),
                                                    ]),
                                                ),
                                                HtmlNode::new_text("\n                "),
                                                HtmlNode::Tag(
                                                    HtmlTag::new("li").contents(vec![
                                                        HtmlNode::new_text("Cream"),
                                                    ]),
                                                ),
                                                HtmlNode::new_text("\n            "),
                                            ])),
                                        HtmlNode::new_text("\n        "),
                                    ]),
                            ),
                            HtmlNode::new_text("\n        "),
                            HtmlNode::Tag(HtmlTag::new("div").classes(vec!["top"]).contents(vec![
                                HtmlNode::Tag(HtmlTag::new("div").classes(vec!["alone"])),
                                HtmlNode::new_text("That div is empty..."),
                            ])),
                            HtmlNode::new_text("\n        "),
                            HtmlNode::Tag(
                                HtmlTag::new("div")
                                    .classes(vec!["tablebox", "shadow"])
                                    .contents(vec![
                                        HtmlNode::new_text("Table of values:\n            "),
                                        HtmlNode::Tag(HtmlTag::new("table").contents(vec![
                                                HtmlNode::new_text("\n                "),
                                                HtmlNode::Tag(HtmlTag::new("tr").contents(vec![
                                                    HtmlNode::new_text("\n                    "),
                                                    HtmlNode::Tag(HtmlTag::new("th").contents(
                                                        vec![HtmlNode::new_text("Company")],
                                                    )),
                                                    HtmlNode::new_text("\n                    "),
                                                    HtmlNode::Tag(HtmlTag::new("th").contents(
                                                        vec![HtmlNode::new_text("Contact")],
                                                    )),
                                                    HtmlNode::new_text("\n                    "),
                                                    HtmlNode::Tag(HtmlTag::new("th").contents(
                                                        vec![HtmlNode::new_text("Country")],
                                                    )),
                                                    HtmlNode::new_text("\n                "),
                                                ])),
                                                HtmlNode::new_text("\n                "),
                                                HtmlNode::Tag(
                                                    HtmlTag::new("tr")
                                                        .ids(vec!["first-data-row"])
                                                        .contents(vec![
                                                            HtmlNode::new_text(
                                                                "\n                    ",
                                                            ),
                                                            HtmlNode::Tag(
                                                                HtmlTag::new("td").contents(vec![
                                                                    HtmlNode::new_text(
                                                                        "Alfreds Futterkiste",
                                                                    ),
                                                                ]),
                                                            ),
                                                            HtmlNode::new_text(
                                                                "\n                    ",
                                                            ),
                                                            HtmlNode::Tag(
                                                                HtmlTag::new("td").contents(vec![
                                                                    HtmlNode::new_text(
                                                                        "Maria Anders",
                                                                    ),
                                                                ]),
                                                            ),
                                                            HtmlNode::new_text(
                                                                "\n                    ",
                                                            ),
                                                            HtmlNode::Tag(
                                                                HtmlTag::new("td").contents(vec![
                                                                    HtmlNode::new_text("Germany"),
                                                                ]),
                                                            ),
                                                            HtmlNode::new_text(
                                                                "\n                ",
                                                            ),
                                                        ]),
                                                ),
                                                HtmlNode::new_text("\n                "),
                                                HtmlNode::Tag(
                                                    HtmlTag::new("tr")
                                                        .attributes(vec![(
                                                            "custom",
                                                            "red",
                                                        )])
                                                        .contents(vec![
                                                            HtmlNode::new_text(
                                                                "\n                    ",
                                                            ),
                                                            HtmlNode::Tag(
                                                                HtmlTag::new("td")
                                                                    .attributes(vec![(
                                                                        "custom", "reds",
                                                                    )])
                                                                    .contents(
                                                                        vec![HtmlNode::new_text(
                                                                "Centro comercial Moctezuma",
                                                            )],
                                                                    ),
                                                            ),
                                                            HtmlNode::new_text(
                                                                "\n                    ",
                                                            ),
                                                            HtmlNode::Tag(
                                                                HtmlTag::new("td")
                                                                    .attributes(vec![(
                                                                        "custom", "re-ds",
                                                                    )])
                                                                    .contents(vec![
                                                                        HtmlNode::new_text(
                                                                            "Francisco Chang",
                                                                        ),
                                                                    ]),
                                                            ),
                                                            HtmlNode::new_text(
                                                                "\n                    ",
                                                            ),
                                                            HtmlNode::Tag(
                                                                HtmlTag::new("td").attributes(vec![(
                                                            "custom",
                                                            "something else",
                                                        )]).contents(vec![
                                                                    HtmlNode::new_text("Mexico"),
                                                                ]),
                                                            ),
                                                            HtmlNode::new_text(
                                                                "\n                ",
                                                            ),
                                                        ]),
                                                ),
                                                HtmlNode::new_text("\n            "),
                                            ])),
                                        HtmlNode::new_text("\n        "),
                                    ]),
                            ),
                            HtmlNode::new_text("\n        "),
                            HtmlNode::Tag(HtmlTag::new("div").contents(vec![
                                HtmlNode::new_text("Form stuffs\n            "),
                                HtmlNode::Tag(HtmlTag::new("form").contents(vec![
                                        HtmlNode::new_text("\n              First name: "),
                                        HtmlNode::Tag(HtmlTag::new("input").attributes(vec![
                                            ("type", "text"),
                                            ("value", "Mickey"),
                                        ])),
                                        HtmlNode::Tag(HtmlTag::new("br")),
                                        HtmlNode::new_text("\n              Last name: "),
                                        HtmlNode::Tag(HtmlTag::new("input").attributes(vec![
                                            ("type", "text"),
                                            ("value", "Mouse"),
                                        ])),
                                        HtmlNode::Tag(HtmlTag::new("br")),
                                        HtmlNode::new_text("\n              Country: "),
                                        HtmlNode::Tag(HtmlTag::new("input").attributes(vec![
                                            ("value", "Disneyland"),
                                            ("disabled", "disabled"),
                                            ("type", "text"),
                                        ])),
                                        HtmlNode::new_text("\n              "),
                                        HtmlNode::Tag(HtmlTag::new("input").attributes(vec![
                                            ("checked", "checked"),
                                            ("type", "radio"),
                                            ("value", "male"),
                                            ("name", "gender"),
                                            ("read-only","read-only")
                                        ])),
                                        HtmlNode::new_text(" Male"),
                                        HtmlNode::Tag(HtmlTag::new("br")),
                                        HtmlNode::new_text("\n              "),
                                        HtmlNode::Tag(HtmlTag::new("input").attributes(vec![
                                            ("type", "radio"),
                                            ("name", "gender"),
                                            ("value", "female"),
                                            ("required", "required"),
                                            ("read-only","false")
                                        ])),
                                        HtmlNode::new_text(" Female"),
                                        HtmlNode::Tag(HtmlTag::new("br")),
                                        HtmlNode::new_text("\n              "),
                                        HtmlNode::Tag(HtmlTag::new("input").attributes(vec![
                                            ("value", "Bike"),
                                            ("checked", "checked"),
                                            ("type", "checkbox"),
                                            ("required","")
                                        ])),
                                        HtmlNode::new_text(" I have a bike"),
                                        HtmlNode::Tag(HtmlTag::new("br")),
                                        HtmlNode::new_text("\n              "),
                                        HtmlNode::Tag(HtmlTag::new("input").attributes(vec![
                                            ("type", "checkbox"),
                                            ("value", "Car"),
                                            ("read-only","")
                                        ])),
                                        HtmlNode::new_text(" I have a car \n            "),
                                    ])),
                                HtmlNode::new_text("\n        "),
                            ])),
                            HtmlNode::new_text("\n    "),
                            HtmlNode::Tag(HtmlTag::new("div").contents(vec![
                                HtmlNode::new_text(" Options\n        "),
                                HtmlNode::Tag(HtmlTag::new("select").contents(vec![
                                            HtmlNode::new_text("\n          "),
                                            HtmlNode::Tag(
                                                HtmlTag::new("option")
                                                    .attributes(vec![
                                                        ("value", "blue"),
                                                        ("selected", ""),
                                                    ])
                                                    .contents(vec![HtmlNode::new_text("Blue")]),
                                            ),
                                            HtmlNode::new_text("\n          "),
                                            HtmlNode::Tag(
                                                HtmlTag::new("option")
                                                    .attributes(vec![
                                                        ("value", "red"),
                                                    ])
                                                    .contents(vec![HtmlNode::new_text("Red")]),
                                            ),
                                            HtmlNode::new_text("\n          "),
                                            HtmlNode::Tag(
                                                HtmlTag::new("option")
                                                    .attributes(vec![
                                                        ("value", "green"),
                                                    ])
                                                    .contents(vec![HtmlNode::new_text("Green")]),
                                            ),
                                            HtmlNode::new_text("\n          "),
                                            HtmlNode::Tag(
                                                HtmlTag::new("option")
                                                    .attributes(vec![
                                                        ("value", "yellow"),
                                                    ])
                                                    .contents(vec![HtmlNode::new_text("Yellow")]),
                                            ),
                                            HtmlNode::new_text("\n          "),
                                        ])),
                                HtmlNode::new_text("\n        "),
                            ])),
                        ])),
                HtmlNode::new_text("\n"),
            ]))],
        };
        assert_eq!(doc, doc_manual);
        let dummy = HtmlNode::Comment(" ".to_owned());
        //test the following match cases:
        // - Tags
        assert_eq!(
            doc.find("p").nodes(),
            vec![&HtmlNode::Tag(HtmlTag::new("p").contents(vec![
                HtmlNode::new_text("A HTML document for testing purposes.",)
            ])),]
        );
        // - Class
        assert_eq!(
            doc.find(".tablebox").nodes(),
            vec![&HtmlNode::Tag(
                HtmlTag::new("div")
                    .classes(vec!["tablebox", "shadow"])
                    .contents(vec![
                        HtmlNode::new_text("Table of values:\n            "),
                        HtmlNode::Tag(HtmlTag::new("table").contents(vec![
                                                HtmlNode::new_text("\n                "),
                                                HtmlNode::Tag(HtmlTag::new("tr").contents(vec![
                                                    HtmlNode::new_text("\n                    "),
                                                    HtmlNode::Tag(HtmlTag::new("th").contents(
                                                        vec![HtmlNode::new_text("Company")],
                                                    )),
                                                    HtmlNode::new_text("\n                    "),
                                                    HtmlNode::Tag(HtmlTag::new("th").contents(
                                                        vec![HtmlNode::new_text("Contact")],
                                                    )),
                                                    HtmlNode::new_text("\n                    "),
                                                    HtmlNode::Tag(HtmlTag::new("th").contents(
                                                        vec![HtmlNode::new_text("Country")],
                                                    )),
                                                    HtmlNode::new_text("\n                "),
                                                ])),
                                                HtmlNode::new_text("\n                "),
                                                HtmlNode::Tag(
                                                    HtmlTag::new("tr")
                                                        .ids(vec!["first-data-row"])
                                                        .contents(vec![
                                                            HtmlNode::new_text(
                                                                "\n                    ",
                                                            ),
                                                            HtmlNode::Tag(
                                                                HtmlTag::new("td").contents(vec![
                                                                    HtmlNode::new_text(
                                                                        "Alfreds Futterkiste",
                                                                    ),
                                                                ]),
                                                            ),
                                                            HtmlNode::new_text(
                                                                "\n                    ",
                                                            ),
                                                            HtmlNode::Tag(
                                                                HtmlTag::new("td").contents(vec![
                                                                    HtmlNode::new_text(
                                                                        "Maria Anders",
                                                                    ),
                                                                ]),
                                                            ),
                                                            HtmlNode::new_text(
                                                                "\n                    ",
                                                            ),
                                                            HtmlNode::Tag(
                                                                HtmlTag::new("td").contents(vec![
                                                                    HtmlNode::new_text("Germany"),
                                                                ]),
                                                            ),
                                                            HtmlNode::new_text(
                                                                "\n                ",
                                                            ),
                                                        ]),
                                                ),
                                                HtmlNode::new_text("\n                "),
                                                HtmlNode::Tag(
                                                    HtmlTag::new("tr")
                                                        .attributes(vec![(
                                                            "custom".to_owned(),
                                                            "red".to_owned(),
                                                        )])
                                                        .contents(vec![
                                                            HtmlNode::new_text(
                                                                "\n                    ",
                                                            ),
                                                            HtmlNode::Tag(
                                                                HtmlTag::new("td")
                                                                    .attributes(vec![(
                                                                        "custom", "reds",
                                                                    )])
                                                                    .contents(
                                                                        vec![HtmlNode::new_text(
                                                                "Centro comercial Moctezuma",
                                                            )],
                                                                    ),
                                                            ),
                                                            HtmlNode::new_text(
                                                                "\n                    ",
                                                            ),
                                                            HtmlNode::Tag(
                                                                HtmlTag::new("td")
                                                                    .attributes(vec![(
                                                                        "custom", "re-ds",
                                                                    )])
                                                                    .contents(vec![
                                                                        HtmlNode::new_text(
                                                                            "Francisco Chang",
                                                                        ),
                                                                    ]),
                                                            ),
                                                            HtmlNode::new_text(
                                                                "\n                    ",
                                                            ),
                                                            HtmlNode::Tag(
                                                                HtmlTag::new("td").contents(vec![
                                                                    HtmlNode::new_text("Mexico"),
                                                                ]),
                                                            ),
                                                            HtmlNode::new_text(
                                                                "\n                ",
                                                            ),
                                                        ]),
                                                ),
                                                HtmlNode::new_text("\n            "),
                                            ]),),
                        HtmlNode::new_text("\n        "),
                    ]),
            ),]
        );
        // - Id
        assert_eq!(
            doc.find("#first-data-row").nodes(),
            vec![&HtmlNode::Tag(
                HtmlTag::new("tr")
                    .ids(vec!["first-data-row"])
                    .contents(vec![
                        HtmlNode::new_text("\n                    ",),
                        HtmlNode::Tag(
                            HtmlTag::new("td")
                                .contents(vec![HtmlNode::new_text("Alfreds Futterkiste",),]),
                        ),
                        HtmlNode::new_text("\n                    ",),
                        HtmlNode::Tag(
                            HtmlTag::new("td").contents(vec![HtmlNode::new_text("Maria Anders",),]),
                        ),
                        HtmlNode::new_text("\n                    ",),
                        HtmlNode::Tag(
                            HtmlTag::new("td").contents(vec![HtmlNode::new_text("Germany"),]),
                        ),
                        HtmlNode::new_text("\n                ",),
                    ]),
            ),]
        );
        // - Attributes:
        //    - Present
        assert_eq!(
            doc.find("[custom]").nodes(),
            vec![
                &HtmlNode::Tag(
                    HtmlTag::new("tr")
                        .attributes(vec![("custom", "red",)])
                        .contents(vec![
                            HtmlNode::new_text("\n                    ",),
                            HtmlNode::Tag(
                                HtmlTag::new("td")
                                    .attributes(vec![("custom", "reds",)])
                                    .contents(vec![HtmlNode::new_text(
                                        "Centro comercial Moctezuma",
                                    )],),
                            ),
                            HtmlNode::new_text("\n                    ",),
                            HtmlNode::Tag(
                                HtmlTag::new("td")
                                    .attributes(vec![("custom", "re-ds",)])
                                    .contents(vec![HtmlNode::new_text("Francisco Chang",),]),
                            ),
                            HtmlNode::new_text("\n                    ",),
                            HtmlNode::Tag(
                                HtmlTag::new("td")
                                    .attributes(vec![("custom", "something else",)])
                                    .contents(vec![HtmlNode::new_text("Mexico"),]),
                            ),
                            HtmlNode::new_text("\n                ",),
                        ]),
                ),
                &HtmlNode::Tag(
                    HtmlTag::new("td")
                        .attributes(vec![("custom", "reds",)])
                        .contents(vec![HtmlNode::new_text("Centro comercial Moctezuma",)],),
                ),
                &HtmlNode::Tag(
                    HtmlTag::new("td")
                        .attributes(vec![("custom", "re-ds",)])
                        .contents(vec![HtmlNode::new_text("Francisco Chang",),]),
                ),
                &HtmlNode::Tag(
                    HtmlTag::new("td")
                        .attributes(vec![("custom", "something else",)])
                        .contents(vec![HtmlNode::new_text("Mexico"),]),
                ),
            ]
        );
        //    - Equal
        assert_eq!(
            doc.find("[custom=red]").nodes(),
            vec![&HtmlNode::Tag(
                HtmlTag::new("tr")
                    .attributes(vec![("custom", "red",)])
                    .contents(vec![
                        HtmlNode::new_text("\n                    ",),
                        HtmlNode::Tag(
                            HtmlTag::new("td")
                                .attributes(vec![("custom", "reds",)])
                                .contents(vec![HtmlNode::new_text("Centro comercial Moctezuma",)],),
                        ),
                        HtmlNode::new_text("\n                    ",),
                        HtmlNode::Tag(
                            HtmlTag::new("td")
                                .attributes(vec![("custom", "re-ds",)])
                                .contents(vec![HtmlNode::new_text("Francisco Chang",),]),
                        ),
                        HtmlNode::new_text("\n                    ",),
                        HtmlNode::Tag(
                            HtmlTag::new("td")
                                .attributes(vec![("custom", "something else",)])
                                .contents(vec![HtmlNode::new_text("Mexico"),]),
                        ),
                        HtmlNode::new_text("\n                ",),
                    ]),
            ),]
        );
        //    - BeginsWith
        assert_eq!(
            doc.find("[custom^=red]").nodes(),
            vec![
                &HtmlNode::Tag(
                    HtmlTag::new("tr")
                        .attributes(vec![("custom", "red",)])
                        .contents(vec![
                            HtmlNode::new_text("\n                    ",),
                            HtmlNode::Tag(
                                HtmlTag::new("td")
                                    .attributes(vec![("custom", "reds",)])
                                    .contents(vec![HtmlNode::new_text(
                                        "Centro comercial Moctezuma",
                                    )],),
                            ),
                            HtmlNode::new_text("\n                    ",),
                            HtmlNode::Tag(
                                HtmlTag::new("td")
                                    .attributes(vec![("custom", "re-ds",)])
                                    .contents(vec![HtmlNode::new_text("Francisco Chang",),]),
                            ),
                            HtmlNode::new_text("\n                    ",),
                            HtmlNode::Tag(
                                HtmlTag::new("td")
                                    .attributes(vec![("custom", "something else",)])
                                    .contents(vec![HtmlNode::new_text("Mexico"),]),
                            ),
                            HtmlNode::new_text("\n                ",),
                        ]),
                ),
                &HtmlNode::Tag(
                    HtmlTag::new("td")
                        .attributes(vec![("custom", "reds",)])
                        .contents(vec![HtmlNode::new_text("Centro comercial Moctezuma",)],),
                ),
            ]
        );
        assert_eq!(
            doc.find("[custom^=re]").nodes(),
            vec![
                &HtmlNode::Tag(
                    HtmlTag::new("tr")
                        .attributes(vec![("custom", "red",)])
                        .contents(vec![
                            HtmlNode::new_text("\n                    ",),
                            HtmlNode::Tag(
                                HtmlTag::new("td")
                                    .attributes(vec![("custom", "reds",)])
                                    .contents(vec![HtmlNode::new_text(
                                        "Centro comercial Moctezuma",
                                    )],),
                            ),
                            HtmlNode::new_text("\n                    ",),
                            HtmlNode::Tag(
                                HtmlTag::new("td")
                                    .attributes(vec![("custom", "re-ds",)])
                                    .contents(vec![HtmlNode::new_text("Francisco Chang",),]),
                            ),
                            HtmlNode::new_text("\n                    ",),
                            HtmlNode::Tag(
                                HtmlTag::new("td")
                                    .attributes(vec![("custom", "something else",)])
                                    .contents(vec![HtmlNode::new_text("Mexico"),]),
                            ),
                            HtmlNode::new_text("\n                ",),
                        ]),
                ),
                &HtmlNode::Tag(
                    HtmlTag::new("td")
                        .attributes(vec![("custom", "reds",)])
                        .contents(vec![HtmlNode::new_text("Centro comercial Moctezuma",)],),
                ),
                &HtmlNode::Tag(
                    HtmlTag::new("td")
                        .attributes(vec![("custom", "re-ds",)])
                        .contents(vec![HtmlNode::new_text("Francisco Chang",),]),
                ),
            ]
        );
        //    - EqualOrBeginsWith
        assert_eq!(
            doc.find("[custom|=red]").nodes(),
            vec![&HtmlNode::Tag(
                HtmlTag::new("tr")
                    .attributes(vec![("custom", "red",)])
                    .contents(vec![
                        HtmlNode::new_text("\n                    ",),
                        HtmlNode::Tag(
                            HtmlTag::new("td")
                                .attributes(vec![("custom", "reds",)])
                                .contents(vec![HtmlNode::new_text("Centro comercial Moctezuma",)],),
                        ),
                        HtmlNode::new_text("\n                    ",),
                        HtmlNode::Tag(
                            HtmlTag::new("td")
                                .attributes(vec![("custom", "re-ds",)])
                                .contents(vec![HtmlNode::new_text("Francisco Chang",),]),
                        ),
                        HtmlNode::new_text("\n                    ",),
                        HtmlNode::Tag(
                            HtmlTag::new("td")
                                .attributes(vec![("custom", "something else",)])
                                .contents(vec![HtmlNode::new_text("Mexico"),]),
                        ),
                        HtmlNode::new_text("\n                ",),
                    ]),
            ),]
        );
        assert_eq!(
            doc.find("[custom|=re]").nodes(),
            vec![&HtmlNode::Tag(
                HtmlTag::new("td")
                    .attributes(vec![("custom", "re-ds",)])
                    .contents(vec![HtmlNode::new_text("Francisco Chang",),]),
            ),]
        );
        //    - EndsWith
        assert_eq!(
            doc.find("[custom$=ds]").nodes(),
            vec![
                &HtmlNode::Tag(
                    HtmlTag::new("td")
                        .attributes(vec![("custom", "reds",)])
                        .contents(vec![HtmlNode::new_text("Centro comercial Moctezuma",)],),
                ),
                &HtmlNode::Tag(
                    HtmlTag::new("td")
                        .attributes(vec![("custom", "re-ds",)])
                        .contents(vec![HtmlNode::new_text("Francisco Chang",),]),
                ),
            ]
        );
        //    - Contains
        assert_eq!(
            doc.find("[custom*=ed]").nodes(),
            vec![
                &HtmlNode::Tag(
                    HtmlTag::new("tr")
                        .attributes(vec![("custom", "red",)])
                        .contents(vec![
                            HtmlNode::new_text("\n                    ",),
                            HtmlNode::Tag(
                                HtmlTag::new("td")
                                    .attributes(vec![("custom", "reds",)])
                                    .contents(vec![HtmlNode::new_text(
                                        "Centro comercial Moctezuma",
                                    )],),
                            ),
                            HtmlNode::new_text("\n                    ",),
                            HtmlNode::Tag(
                                HtmlTag::new("td")
                                    .attributes(vec![("custom", "re-ds",)])
                                    .contents(vec![HtmlNode::new_text("Francisco Chang",),]),
                            ),
                            HtmlNode::new_text("\n                    ",),
                            HtmlNode::Tag(
                                HtmlTag::new("td")
                                    .attributes(vec![("custom", "something else",)])
                                    .contents(vec![HtmlNode::new_text("Mexico"),]),
                            ),
                            HtmlNode::new_text("\n                ",),
                        ]),
                ),
                &HtmlNode::Tag(
                    HtmlTag::new("td")
                        .attributes(vec![("custom", "reds",)])
                        .contents(vec![HtmlNode::new_text("Centro comercial Moctezuma",)],),
                ),
            ]
        );
        //    - ContainsWord
        assert_eq!(
            doc.find("[custom~=something]").nodes(),
            vec![&HtmlNode::Tag(
                HtmlTag::new("td")
                    .attributes(vec![("custom", "something else",)])
                    .contents(vec![HtmlNode::new_text("Mexico"),]),
            ),]
        );
        // - Refiners:
        //    - Checked
        assert_eq!(
            doc.find(":checked").nodes(),
            vec![
                &HtmlNode::Tag(HtmlTag::new("input").attributes(vec![
                    ("checked", "checked"),
                    ("type", "radio"),
                    ("value", "male"),
                    ("name", "gender"),
                    ("read-only", "read-only")
                ])),
                &HtmlNode::Tag(HtmlTag::new("input").attributes(vec![
                    ("value", "Bike"),
                    ("checked", "checked"),
                    ("type", "checkbox"),
                    ("required", "")
                ])),
                &HtmlNode::Tag(
                    HtmlTag::new("option")
                        .attributes(vec![("value", "blue"), ("selected", ""),])
                        .contents(vec![HtmlNode::new_text("Blue")]),
                ),
            ]
        );
        //    - Default
        assert_eq!(
            doc.find(":default").nodes(),
            vec![
                &HtmlNode::Tag(HtmlTag::new("input").attributes(vec![
                    ("checked", "checked"),
                    ("type", "radio"),
                    ("value", "male"),
                    ("name", "gender"),
                    ("read-only", "read-only")
                ])),
                &HtmlNode::Tag(HtmlTag::new("input").attributes(vec![
                    ("value", "Bike"),
                    ("checked", "checked"),
                    ("type", "checkbox"),
                    ("required", "")
                ])),
                &HtmlNode::Tag(
                    HtmlTag::new("option")
                        .attributes(vec![("value", "blue"), ("selected", ""),])
                        .contents(vec![HtmlNode::new_text("Blue")]),
                ),
            ]
        );
        //    - Disabled
        assert_eq!(
            doc.find(":disabled").nodes(),
            vec![&HtmlNode::Tag(HtmlTag::new("input").attributes(vec![
                ("value", "Disneyland"),
                ("type", "text"),
                ("disabled", "disabled")
            ])),]
        );
        //    - Enabled
        assert_eq!(
            doc.find(":enabled").nodes(),
            vec![
                &HtmlNode::Tag(
                    HtmlTag::new("input").attributes(vec![("type", "text"), ("value", "Mickey"),])
                ),
                &HtmlNode::Tag(
                    HtmlTag::new("input").attributes(vec![("type", "text"), ("value", "Mouse"),])
                ),
                &HtmlNode::Tag(HtmlTag::new("input").attributes(vec![
                    ("checked", "checked"),
                    ("type", "radio"),
                    ("value", "male"),
                    ("name", "gender"),
                    ("read-only", "read-only")
                ])),
                &HtmlNode::Tag(HtmlTag::new("input").attributes(vec![
                    ("type", "radio"),
                    ("name", "gender"),
                    ("value", "female"),
                    ("required", "required"),
                    ("read-only", "false")
                ])),
                &HtmlNode::Tag(HtmlTag::new("input").attributes(vec![
                    ("value", "Bike"),
                    ("checked", "checked"),
                    ("type", "checkbox"),
                    ("required", "")
                ])),
                &HtmlNode::Tag(HtmlTag::new("input").attributes(vec![
                    ("type", "checkbox"),
                    ("value", "Car"),
                    ("read-only", "")
                ])),
                &HtmlNode::Tag(HtmlTag::new("select").contents(vec![
                    HtmlNode::new_text("\n          "),
                    HtmlNode::Tag(
                        HtmlTag::new("option")
                            .attributes(vec![("value", "blue"), ("selected", ""),])
                            .contents(vec![HtmlNode::new_text("Blue")]),
                    ),
                    HtmlNode::new_text("\n          "),
                    HtmlNode::Tag(
                        HtmlTag::new("option")
                            .attributes(vec![("value", "red")])
                            .contents(vec![HtmlNode::new_text("Red")]),
                    ),
                    HtmlNode::new_text("\n          "),
                    HtmlNode::Tag(
                        HtmlTag::new("option")
                            .attributes(vec![("value", "green"),])
                            .contents(vec![HtmlNode::new_text("Green")]),
                    ),
                    HtmlNode::new_text("\n          "),
                    HtmlNode::Tag(
                        HtmlTag::new("option")
                            .attributes(vec![("value", "yellow"),])
                            .contents(vec![HtmlNode::new_text("Yellow")]),
                    ),
                    HtmlNode::new_text("\n          "),
                ])),
                &HtmlNode::Tag(
                    HtmlTag::new("option")
                        .attributes(vec![("value", "blue"), ("selected", ""),])
                        .contents(vec![HtmlNode::new_text("Blue")]),
                ),
                &HtmlNode::Tag(
                    HtmlTag::new("option")
                        .attributes(vec![("value", "red"),])
                        .contents(vec![HtmlNode::new_text("Red")]),
                ),
                &HtmlNode::Tag(
                    HtmlTag::new("option")
                        .attributes(vec![("value", "green"),])
                        .contents(vec![HtmlNode::new_text("Green")]),
                ),
                &HtmlNode::Tag(
                    HtmlTag::new("option")
                        .attributes(vec![("value", "yellow"),])
                        .contents(vec![HtmlNode::new_text("Yellow")]),
                ),
            ]
        );
        //    - Optional
        assert_eq!(
            doc.find(":optional").nodes(),
            vec![
                &HtmlNode::Tag(
                    HtmlTag::new("input").attributes(vec![("type", "text"), ("value", "Mickey"),])
                ),
                &HtmlNode::Tag(
                    HtmlTag::new("input").attributes(vec![("type", "text"), ("value", "Mouse"),])
                ),
                &HtmlNode::Tag(HtmlTag::new("input").attributes(vec![
                    ("value", "Disneyland"),
                    ("disabled", "disabled"),
                    ("type", "text"),
                ])),
                &HtmlNode::Tag(HtmlTag::new("input").attributes(vec![
                    ("checked", "checked"),
                    ("type", "radio"),
                    ("value", "male"),
                    ("name", "gender"),
                    ("read-only", "read-only")
                ])),
                &HtmlNode::Tag(HtmlTag::new("input").attributes(vec![
                    ("type", "checkbox"),
                    ("value", "Car"),
                    ("read-only", "")
                ])),
                &HtmlNode::Tag(HtmlTag::new("select").contents(vec![
                            HtmlNode::new_text("\n          "),
                            HtmlNode::Tag(
                                HtmlTag::new("option")
                                    .attributes(vec![("value", "blue"), ("selected", ""),])
                                    .contents(vec![HtmlNode::new_text("Blue")]),
                            ),
                            HtmlNode::new_text("\n          "),
                            HtmlNode::Tag(
                                HtmlTag::new("option")
                                    .attributes(vec![("value", "red"),])
                                    .contents(vec![HtmlNode::new_text("Red")]),
                            ),
                            HtmlNode::new_text("\n          "),
                            HtmlNode::Tag(
                                HtmlTag::new("option")
                                    .attributes(vec![("value", "green"),])
                                    .contents(vec![HtmlNode::new_text("Green")]),
                            ),
                            HtmlNode::new_text("\n          "),
                            HtmlNode::Tag(
                                HtmlTag::new("option")
                                    .attributes(vec![("value", "yellow"),])
                                    .contents(vec![HtmlNode::new_text("Yellow")]),
                            ),
                            HtmlNode::new_text("\n          "),
                        ])),
            ]
        );
        //    - Required
        assert_eq!(
            doc.find(":required").nodes(),
            vec![
                &HtmlNode::Tag(HtmlTag::new("input").attributes(vec![
                    ("type", "radio"),
                    ("name", "gender"),
                    ("value", "female"),
                    ("required", "required"),
                    ("read-only", "false")
                ])),
                &HtmlNode::Tag(HtmlTag::new("input").attributes(vec![
                    ("value", "Bike"),
                    ("checked", "checked"),
                    ("type", "checkbox"),
                    ("required", "")
                ])),
            ]
        );
        //    - ReadOnly
        assert_eq!(
            doc.find("input:read-only").nodes(),
            vec![
                &HtmlNode::Tag(HtmlTag::new("input").attributes(vec![
                    ("checked", "checked"),
                    ("type", "radio"),
                    ("value", "male"),
                    ("name", "gender"),
                    ("read-only", "read-only")
                ])),
                &HtmlNode::Tag(HtmlTag::new("input").attributes(vec![
                    ("type", "checkbox"),
                    ("value", "Car"),
                    ("read-only", "")
                ])),
            ]
        );
        //    - ReadWrite
        assert_eq!(
            doc.find("input:read-write").nodes(),
            vec![
                &HtmlNode::Tag(
                    HtmlTag::new("input").attributes(vec![("type", "text"), ("value", "Mickey"),])
                ),
                &HtmlNode::Tag(
                    HtmlTag::new("input").attributes(vec![("type", "text"), ("value", "Mouse"),])
                ),
                &HtmlNode::Tag(HtmlTag::new("input").attributes(vec![
                    ("value", "Disneyland"),
                    ("disabled", "disabled"),
                    ("type", "text"),
                ])),
                &HtmlNode::Tag(HtmlTag::new("input").attributes(vec![
                    ("type", "radio"),
                    ("name", "gender"),
                    ("value", "female"),
                    ("required", "required"),
                    ("read-only", "false")
                ])),
                &HtmlNode::Tag(HtmlTag::new("input").attributes(vec![
                    ("value", "Bike"),
                    ("checked", "checked"),
                    ("type", "checkbox"),
                    ("required", "")
                ])),
            ]
        );
        //    - Empty
        assert_eq!(
            doc.find("div:empty").nodes(),
            vec![&HtmlNode::Tag(HtmlTag::new("div").classes(vec!["alone"]))]
        );
        //    - FirstChild
        assert_eq!(
            doc.find("option:first-child").nodes(),
            vec![&HtmlNode::Tag(
                HtmlTag::new("option")
                    .attributes(vec![("value", "blue"), ("selected", ""),])
                    .contents(vec![HtmlNode::new_text("Blue")]),
            ),]
        );
        //    - LastChild
        assert_eq!(
            doc.find("option:last-child").nodes(),
            vec![&HtmlNode::Tag(
                HtmlTag::new("option")
                    .attributes(vec![("value", "yellow"),])
                    .contents(vec![HtmlNode::new_text("Yellow")]),
            ),]
        );
        //    - NthChild (exact/functional)
        assert_eq!(
            doc.find("option:nth-child(2)").nodes(),
            vec![&HtmlNode::Tag(
                HtmlTag::new("option")
                    .attributes(vec![("value", "red"),])
                    .contents(vec![HtmlNode::new_text("Red")]),
            ),]
        );
        assert_eq!(
            doc.find("option:nth-child(2n+1)").nodes(),
            vec![
                &HtmlNode::Tag(
                    HtmlTag::new("option")
                        .attributes(vec![("value", "blue"), ("selected", ""),])
                        .contents(vec![HtmlNode::new_text("Blue")]),
                ),
                &HtmlNode::Tag(
                    HtmlTag::new("option")
                        .attributes(vec![("value", "green"),])
                        .contents(vec![HtmlNode::new_text("Green")]),
                ),
            ]
        );
        assert_eq!(
            doc.find("option:nth-child(-2n+3)").nodes(),
            vec![
                &HtmlNode::Tag(
                    HtmlTag::new("option")
                        .attributes(vec![("value", "blue"), ("selected", ""),])
                        .contents(vec![HtmlNode::new_text("Blue")]),
                ),
                &HtmlNode::Tag(
                    HtmlTag::new("option")
                        .attributes(vec![("value", "green"),])
                        .contents(vec![HtmlNode::new_text("Green")]),
                ),
            ]
        );
        assert_eq!(
            doc.find("option:nth-child(odd)").nodes(),
            vec![
                &HtmlNode::Tag(
                    HtmlTag::new("option")
                        .attributes(vec![("value", "blue"), ("selected", ""),])
                        .contents(vec![HtmlNode::new_text("Blue")]),
                ),
                &HtmlNode::Tag(
                    HtmlTag::new("option")
                        .attributes(vec![("value", "green"),])
                        .contents(vec![HtmlNode::new_text("Green")]),
                ),
            ]
        );
        assert_eq!(
            doc.find("option:nth-child(even)").nodes(),
            vec![
                &HtmlNode::Tag(
                    HtmlTag::new("option")
                        .attributes(vec![("value", "red"),])
                        .contents(vec![HtmlNode::new_text("Red")]),
                ),
                &HtmlNode::Tag(
                    HtmlTag::new("option")
                        .attributes(vec![("value", "yellow"),])
                        .contents(vec![HtmlNode::new_text("Yellow")]),
                ),
            ]
        );
        //    - NthLastChild (exact/functional)
        assert_eq!(
            doc.find("option:nth-last-child(2)").nodes(),
            vec![&HtmlNode::Tag(
                HtmlTag::new("option")
                    .attributes(vec![("value", "green"),])
                    .contents(vec![HtmlNode::new_text("Green")]),
            ),]
        );
        assert_eq!(
            doc.find("option:nth-last-child(2n+1)").nodes(),
            vec![
                &HtmlNode::Tag(
                    HtmlTag::new("option")
                        .attributes(vec![("value", "red"),])
                        .contents(vec![HtmlNode::new_text("Red")]),
                ),
                &HtmlNode::Tag(
                    HtmlTag::new("option")
                        .attributes(vec![("value", "yellow"),])
                        .contents(vec![HtmlNode::new_text("Yellow")]),
                ),
            ]
        );
        assert_eq!(
            doc.find("option:nth-last-child(-2n+3)").nodes(),
            vec![
                &HtmlNode::Tag(
                    HtmlTag::new("option")
                        .attributes(vec![("value", "red"),])
                        .contents(vec![HtmlNode::new_text("Red")]),
                ),
                &HtmlNode::Tag(
                    HtmlTag::new("option")
                        .attributes(vec![("value", "yellow"),])
                        .contents(vec![HtmlNode::new_text("Yellow")]),
                ),
            ]
        );
        assert_eq!(
            doc.find("option:nth-last-child(odd)").nodes(),
            vec![
                &HtmlNode::Tag(
                    HtmlTag::new("option")
                        .attributes(vec![("value", "red"),])
                        .contents(vec![HtmlNode::new_text("Red")]),
                ),
                &HtmlNode::Tag(
                    HtmlTag::new("option")
                        .attributes(vec![("value", "yellow"),])
                        .contents(vec![HtmlNode::new_text("Yellow")]),
                ),
            ]
        );
        assert_eq!(
            doc.find("option:nth-last-child(even)").nodes(),
            vec![
                &HtmlNode::Tag(
                    HtmlTag::new("option")
                        .attributes(vec![("value", "blue"), ("selected", ""),])
                        .contents(vec![HtmlNode::new_text("Blue")]),
                ),
                &HtmlNode::Tag(
                    HtmlTag::new("option")
                        .attributes(vec![("value", "green"),])
                        .contents(vec![HtmlNode::new_text("Green")]),
                ),
            ]
        );
        //    - OnlyChild
        assert_eq!(
            doc.find("div:only-child").nodes(),
            vec![&HtmlNode::Tag(HtmlTag::new("div").classes(vec!["alone"]))]
        );
        //    - FirstOfType
        assert_eq!(
            doc.find("option:first-of-type").nodes(),
            vec![&HtmlNode::Tag(
                HtmlTag::new("option")
                    .attributes(vec![("value", "blue"), ("selected", ""),])
                    .contents(vec![HtmlNode::new_text("Blue")]),
            ),]
        );
        //    - LastOfType
        assert_eq!(
            doc.find("option:last-of-type").nodes(),
            vec![&HtmlNode::Tag(
                HtmlTag::new("option")
                    .attributes(vec![("value", "yellow"),])
                    .contents(vec![HtmlNode::new_text("Yellow")]),
            ),]
        );
        //    - NthOfType (exact/functional)
        assert_eq!(
            doc.find("option:nth-of-type(2)").nodes(),
            vec![&HtmlNode::Tag(
                HtmlTag::new("option")
                    .attributes(vec![("value", "red"),])
                    .contents(vec![HtmlNode::new_text("Red")]),
            ),]
        );
        assert_eq!(
            doc.find("option:nth-of-type(2n+1)").nodes(),
            vec![
                &HtmlNode::Tag(
                    HtmlTag::new("option")
                        .attributes(vec![("value", "blue"), ("selected", ""),])
                        .contents(vec![HtmlNode::new_text("Blue")]),
                ),
                &HtmlNode::Tag(
                    HtmlTag::new("option")
                        .attributes(vec![("value", "green"),])
                        .contents(vec![HtmlNode::new_text("Green")]),
                ),
            ]
        );
        assert_eq!(
            doc.find("option:nth-of-type(-2n+3)").nodes(),
            vec![
                &HtmlNode::Tag(
                    HtmlTag::new("option")
                        .attributes(vec![("value", "blue"), ("selected", ""),])
                        .contents(vec![HtmlNode::new_text("Blue")]),
                ),
                &HtmlNode::Tag(
                    HtmlTag::new("option")
                        .attributes(vec![("value", "green"),])
                        .contents(vec![HtmlNode::new_text("Green")]),
                ),
            ]
        );
        assert_eq!(
            doc.find("option:nth-of-type(odd)").nodes(),
            vec![
                &HtmlNode::Tag(
                    HtmlTag::new("option")
                        .attributes(vec![("value", "blue"), ("selected", ""),])
                        .contents(vec![HtmlNode::new_text("Blue")]),
                ),
                &HtmlNode::Tag(
                    HtmlTag::new("option")
                        .attributes(vec![("value", "green"),])
                        .contents(vec![HtmlNode::new_text("Green")]),
                ),
            ]
        );
        assert_eq!(
            doc.find("option:nth-of-type(even)").nodes(),
            vec![
                &HtmlNode::Tag(
                    HtmlTag::new("option")
                        .attributes(vec![("value", "red"),])
                        .contents(vec![HtmlNode::new_text("Red")]),
                ),
                &HtmlNode::Tag(
                    HtmlTag::new("option")
                        .attributes(vec![("value", "yellow"),])
                        .contents(vec![HtmlNode::new_text("Yellow")]),
                ),
            ]
        );
        //    - NthLastOfType (exact/functional)
        assert_eq!(
            doc.find("option:nth-last-of-type(2)").nodes(),
            vec![&HtmlNode::Tag(
                HtmlTag::new("option")
                    .attributes(vec![("value", "green"),])
                    .contents(vec![HtmlNode::new_text("Green")]),
            ),]
        );
        assert_eq!(
            doc.find("option:nth-last-of-type(2n+1)").nodes(),
            vec![
                &HtmlNode::Tag(
                    HtmlTag::new("option")
                        .attributes(vec![("value", "red"),])
                        .contents(vec![HtmlNode::new_text("Red")]),
                ),
                &HtmlNode::Tag(
                    HtmlTag::new("option")
                        .attributes(vec![("value", "yellow"),])
                        .contents(vec![HtmlNode::new_text("Yellow")]),
                ),
            ]
        );
        assert_eq!(
            doc.find("option:nth-last-of-type(-2n+3)").nodes(),
            vec![
                &HtmlNode::Tag(
                    HtmlTag::new("option")
                        .attributes(vec![("value", "red"),])
                        .contents(vec![HtmlNode::new_text("Red")]),
                ),
                &HtmlNode::Tag(
                    HtmlTag::new("option")
                        .attributes(vec![("value", "yellow"),])
                        .contents(vec![HtmlNode::new_text("Yellow")]),
                ),
            ]
        );
        assert_eq!(
            doc.find("option:nth-last-of-type(odd)").nodes(),
            vec![
                &HtmlNode::Tag(
                    HtmlTag::new("option")
                        .attributes(vec![("value", "red"),])
                        .contents(vec![HtmlNode::new_text("Red")]),
                ),
                &HtmlNode::Tag(
                    HtmlTag::new("option")
                        .attributes(vec![("value", "yellow"),])
                        .contents(vec![HtmlNode::new_text("Yellow")]),
                ),
            ]
        );
        assert_eq!(
            doc.find("option:nth-last-of-type(even)").nodes(),
            vec![
                &HtmlNode::Tag(
                    HtmlTag::new("option")
                        .attributes(vec![("value", "blue"), ("selected", ""),])
                        .contents(vec![HtmlNode::new_text("Blue")]),
                ),
                &HtmlNode::Tag(
                    HtmlTag::new("option")
                        .attributes(vec![("value", "green"),])
                        .contents(vec![HtmlNode::new_text("Green")]),
                ),
            ]
        );
        //    - OnlyOfType
        assert_eq!(
            doc.find("p:only-of-type").nodes(),
            vec![&HtmlNode::Tag(HtmlTag::new("p").contents(vec![
                HtmlNode::new_text("A HTML document for testing purposes.",)
            ])),]
        );
        //    - Not
        assert_eq!(
            doc.find("tr:not(#first-data-row)").nodes(),
            vec![
                &HtmlNode::Tag(HtmlTag::new("tr").contents(vec![
                    HtmlNode::new_text("\n                    "),
                    HtmlNode::Tag(
                        HtmlTag::new("th").contents(vec![HtmlNode::new_text("Company")],)
                    ),
                    HtmlNode::new_text("\n                    "),
                    HtmlNode::Tag(
                        HtmlTag::new("th").contents(vec![HtmlNode::new_text("Contact")],)
                    ),
                    HtmlNode::new_text("\n                    "),
                    HtmlNode::Tag(
                        HtmlTag::new("th").contents(vec![HtmlNode::new_text("Country")],)
                    ),
                    HtmlNode::new_text("\n                "),
                ])),
                &HtmlNode::Tag(
                    HtmlTag::new("tr")
                        .attributes(vec![("custom", "red",)])
                        .contents(vec![
                            HtmlNode::new_text("\n                    ",),
                            HtmlNode::Tag(
                                HtmlTag::new("td")
                                    .attributes(vec![("custom", "reds",)])
                                    .contents(vec![HtmlNode::new_text(
                                        "Centro comercial Moctezuma",
                                    )],),
                            ),
                            HtmlNode::new_text("\n                    ",),
                            HtmlNode::Tag(
                                HtmlTag::new("td")
                                    .attributes(vec![("custom", "re-ds",)])
                                    .contents(vec![HtmlNode::new_text("Francisco Chang",),]),
                            ),
                            HtmlNode::new_text("\n                    ",),
                            HtmlNode::Tag(
                                HtmlTag::new("td")
                                    .attributes(vec![("custom", "something else",)])
                                    .contents(vec![HtmlNode::new_text("Mexico"),]),
                            ),
                            HtmlNode::new_text("\n                ",),
                        ]),
                ),
            ]
        );
        //    - Root
        assert_eq!(
            doc.find(":root").nodes(),
            vec![&HtmlNode::Tag(HtmlTag::new("html").contents(vec![
                HtmlNode::new_text("\n    "),
                HtmlNode::Tag(HtmlTag::new("head").contents(vec![
                        HtmlNode::new_text("\n        "),
                        HtmlNode::Tag(HtmlTag::new("meta").attributes(vec![
                            ("content", "text/html; charset=UTF-8"),
                            ("http-equiv", "Content-Type"),
                        ])),
                        HtmlNode::new_text("\n        "),
                        HtmlNode::Tag(
                            HtmlTag::new("title").contents(vec![HtmlNode::Text(
                                "A Sample HTML Document".to_owned(),
                            )]),
                        ),
                        HtmlNode::new_text("\n        "),
                        HtmlNode::Tag(HtmlTag::new("meta").attributes(vec![
                            ("name", "description"),
                            ("content", "A HTML document for testing purposes."),
                        ])),
                        HtmlNode::new_text("\n        "),
                        HtmlNode::Tag(HtmlTag::new("meta").attributes(vec![
                            ("name".to_owned(), "author".to_owned()),
                            ("content".to_owned(), "HB".to_owned()),
                        ])),
                        HtmlNode::new_text("\n        "),
                        HtmlNode::Tag(HtmlTag::new("meta").attributes(vec![
                            ("name", "viewport"),
                            ("content", "width=device-width, initial-scale=1"),
                        ])),
                        HtmlNode::new_text("\n    "),
                    ])),
                HtmlNode::new_text("\n    "),
                HtmlNode::Tag(HtmlTag::new("body").contents(vec![
                            HtmlNode::new_text("\n        "),
                            HtmlNode::Tag(
                                HtmlTag::new("h1")
                                    .classes(vec!["heading"])
                                    .contents(vec![HtmlNode::new_text("Test HTML Document")]),
                            ),
                            HtmlNode::new_text("\n        "),
                            HtmlNode::Tag(HtmlTag::new("p").contents(vec![HtmlNode::new_text(
                                "A HTML document for testing purposes.",
                            )])),
                            HtmlNode::new_text("\n        "),
                            HtmlNode::Tag(
                                HtmlTag::new("div")
                                    .classes(vec!["listbox", "shadow"])
                                    .contents(vec![
                                        HtmlNode::new_text("List of dairy items:\n            "),
                                        HtmlNode::Tag(HtmlTag::new("ul").contents(vec![
                                                HtmlNode::new_text("\n                "),
                                                HtmlNode::Tag(
                                                    HtmlTag::new("li")
                                                        .contents(vec![HtmlNode::new_text("Milk")]),
                                                ),
                                                HtmlNode::new_text("\n                "),
                                                HtmlNode::Tag(
                                                    HtmlTag::new("li").contents(vec![
                                                        HtmlNode::new_text("Cheese"),
                                                    ]),
                                                ),
                                                HtmlNode::new_text("\n                "),
                                                HtmlNode::Tag(
                                                    HtmlTag::new("li").contents(vec![
                                                        HtmlNode::new_text("Yoghurt"),
                                                    ]),
                                                ),
                                                HtmlNode::new_text("\n                "),
                                                HtmlNode::Tag(
                                                    HtmlTag::new("li").contents(vec![
                                                        HtmlNode::new_text("Cream"),
                                                    ]),
                                                ),
                                                HtmlNode::new_text("\n            "),
                                            ])),
                                        HtmlNode::new_text("\n        "),
                                    ]),
                            ),
                            HtmlNode::new_text("\n        "),
                            HtmlNode::Tag(HtmlTag::new("div").classes(vec!["top"]).contents(vec![
                                HtmlNode::Tag(HtmlTag::new("div").classes(vec!["alone"])),
                                HtmlNode::new_text("That div is empty..."),
                            ])),
                            HtmlNode::new_text("\n        "),
                            HtmlNode::Tag(
                                HtmlTag::new("div")
                                    .classes(vec!["tablebox", "shadow"])
                                    .contents(vec![
                                        HtmlNode::new_text("Table of values:\n            "),
                                        HtmlNode::Tag(HtmlTag::new("table").contents(vec![
                                                HtmlNode::new_text("\n                "),
                                                HtmlNode::Tag(HtmlTag::new("tr").contents(vec![
                                                    HtmlNode::new_text("\n                    "),
                                                    HtmlNode::Tag(HtmlTag::new("th").contents(
                                                        vec![HtmlNode::new_text("Company")],
                                                    )),
                                                    HtmlNode::new_text("\n                    "),
                                                    HtmlNode::Tag(HtmlTag::new("th").contents(
                                                        vec![HtmlNode::new_text("Contact")],
                                                    )),
                                                    HtmlNode::new_text("\n                    "),
                                                    HtmlNode::Tag(HtmlTag::new("th").contents(
                                                        vec![HtmlNode::new_text("Country")],
                                                    )),
                                                    HtmlNode::new_text("\n                "),
                                                ])),
                                                HtmlNode::new_text("\n                "),
                                                HtmlNode::Tag(
                                                    HtmlTag::new("tr")
                                                        .ids(vec!["first-data-row"])
                                                        .contents(vec![
                                                            HtmlNode::new_text(
                                                                "\n                    ",
                                                            ),
                                                            HtmlNode::Tag(
                                                                HtmlTag::new("td").contents(vec![
                                                                    HtmlNode::new_text(
                                                                        "Alfreds Futterkiste",
                                                                    ),
                                                                ]),
                                                            ),
                                                            HtmlNode::new_text(
                                                                "\n                    ",
                                                            ),
                                                            HtmlNode::Tag(
                                                                HtmlTag::new("td").contents(vec![
                                                                    HtmlNode::new_text(
                                                                        "Maria Anders",
                                                                    ),
                                                                ]),
                                                            ),
                                                            HtmlNode::new_text(
                                                                "\n                    ",
                                                            ),
                                                            HtmlNode::Tag(
                                                                HtmlTag::new("td").contents(vec![
                                                                    HtmlNode::new_text("Germany"),
                                                                ]),
                                                            ),
                                                            HtmlNode::new_text(
                                                                "\n                ",
                                                            ),
                                                        ]),
                                                ),
                                                HtmlNode::new_text("\n                "),
                                                HtmlNode::Tag(
                                                    HtmlTag::new("tr")
                                                        .attributes(vec![(
                                                            "custom",
                                                            "red",
                                                        )])
                                                        .contents(vec![
                                                            HtmlNode::new_text(
                                                                "\n                    ",
                                                            ),
                                                            HtmlNode::Tag(
                                                                HtmlTag::new("td")
                                                                    .attributes(vec![(
                                                                        "custom", "reds",
                                                                    )])
                                                                    .contents(
                                                                        vec![HtmlNode::new_text(
                                                                "Centro comercial Moctezuma",
                                                            )],
                                                                    ),
                                                            ),
                                                            HtmlNode::new_text(
                                                                "\n                    ",
                                                            ),
                                                            HtmlNode::Tag(
                                                                HtmlTag::new("td")
                                                                    .attributes(vec![(
                                                                        "custom", "re-ds",
                                                                    )])
                                                                    .contents(vec![
                                                                        HtmlNode::new_text(
                                                                            "Francisco Chang",
                                                                        ),
                                                                    ]),
                                                            ),
                                                            HtmlNode::new_text(
                                                                "\n                    ",
                                                            ),
                                                            HtmlNode::Tag(
                                                                HtmlTag::new("td").attributes(vec![(
                                                            "custom",
                                                            "something else",
                                                        )]).contents(vec![
                                                                    HtmlNode::new_text("Mexico"),
                                                                ]),
                                                            ),
                                                            HtmlNode::new_text(
                                                                "\n                ",
                                                            ),
                                                        ]),
                                                ),
                                                HtmlNode::new_text("\n            "),
                                            ])),
                                        HtmlNode::new_text("\n        "),
                                    ]),
                            ),
                            HtmlNode::new_text("\n        "),
                            HtmlNode::Tag(HtmlTag::new("div").contents(vec![
                                HtmlNode::new_text("Form stuffs\n            "),
                                HtmlNode::Tag(HtmlTag::new("form").contents(vec![
                                        HtmlNode::new_text("\n              First name: "),
                                        HtmlNode::Tag(HtmlTag::new("input").attributes(vec![
                                            ("type", "text"),
                                            ("value", "Mickey"),
                                        ])),
                                        HtmlNode::Tag(HtmlTag::new("br")),
                                        HtmlNode::new_text("\n              Last name: "),
                                        HtmlNode::Tag(HtmlTag::new("input").attributes(vec![
                                            ("type", "text"),
                                            ("value", "Mouse"),
                                        ])),
                                        HtmlNode::Tag(HtmlTag::new("br")),
                                        HtmlNode::new_text("\n              Country: "),
                                        HtmlNode::Tag(HtmlTag::new("input").attributes(vec![
                                            ("value", "Disneyland"),
                                            ("disabled", "disabled"),
                                            ("type", "text"),
                                        ])),
                                        HtmlNode::new_text("\n              "),
                                        HtmlNode::Tag(HtmlTag::new("input").attributes(vec![
                                            ("checked", "checked"),
                                            ("type", "radio"),
                                            ("value", "male"),
                                            ("name", "gender"),
                                            ("read-only","read-only")
                                        ])),
                                        HtmlNode::new_text(" Male"),
                                        HtmlNode::Tag(HtmlTag::new("br")),
                                        HtmlNode::new_text("\n              "),
                                        HtmlNode::Tag(HtmlTag::new("input").attributes(vec![
                                            ("type", "radio"),
                                            ("name", "gender"),
                                            ("value", "female"),
                                            ("required", "required"),
                                            ("read-only","false")
                                        ])),
                                        HtmlNode::new_text(" Female"),
                                        HtmlNode::Tag(HtmlTag::new("br")),
                                        HtmlNode::new_text("\n              "),
                                        HtmlNode::Tag(HtmlTag::new("input").attributes(vec![
                                            ("value", "Bike"),
                                            ("checked", "checked"),
                                            ("type", "checkbox"),
                                            ("required","")
                                        ])),
                                        HtmlNode::new_text(" I have a bike"),
                                        HtmlNode::Tag(HtmlTag::new("br")),
                                        HtmlNode::new_text("\n              "),
                                        HtmlNode::Tag(HtmlTag::new("input").attributes(vec![
                                            ("type", "checkbox"),
                                            ("value", "Car"),
                                            ("read-only","")
                                        ])),
                                        HtmlNode::new_text(" I have a car \n            "),
                                    ])),
                                HtmlNode::new_text("\n        "),
                            ])),
                            HtmlNode::new_text("\n    "),
                            HtmlNode::Tag(HtmlTag::new("div").contents(vec![
                                HtmlNode::new_text(" Options\n        "),
                                HtmlNode::Tag(HtmlTag::new("select").contents(vec![
                                            HtmlNode::new_text("\n          "),
                                            HtmlNode::Tag(
                                                HtmlTag::new("option")
                                                    .attributes(vec![
                                                        ("value", "blue"),
                                                        ("selected", ""),
                                                    ])
                                                    .contents(vec![HtmlNode::new_text("Blue")]),
                                            ),
                                            HtmlNode::new_text("\n          "),
                                            HtmlNode::Tag(
                                                HtmlTag::new("option")
                                                    .attributes(vec![
                                                        ("value", "red"),
                                                    ])
                                                    .contents(vec![HtmlNode::new_text("Red")]),
                                            ),
                                            HtmlNode::new_text("\n          "),
                                            HtmlNode::Tag(
                                                HtmlTag::new("option")
                                                    .attributes(vec![
                                                        ("value", "green"),
                                                    ])
                                                    .contents(vec![HtmlNode::new_text("Green")]),
                                            ),
                                            HtmlNode::new_text("\n          "),
                                            HtmlNode::Tag(
                                                HtmlTag::new("option")
                                                    .attributes(vec![
                                                        ("value", "yellow"),
                                                    ])
                                                    .contents(vec![HtmlNode::new_text("Yellow")]),
                                            ),
                                            HtmlNode::new_text("\n          "),
                                        ])),
                                HtmlNode::new_text("\n        "),
                            ])),
                        ])),
                HtmlNode::new_text("\n"),
            ]))]
        );
        // Parent
        assert_eq!(
            doc.find("body > p").nodes(),
            vec![&HtmlNode::Tag(HtmlTag::new("p").contents(vec![
                HtmlNode::new_text("A HTML document for testing purposes.",)
            ])),]
        );
        // Ancestor
        assert_eq!(
            doc.find("div td[custom]").nodes(),
            vec![
                &HtmlNode::Tag(
                    HtmlTag::new("td")
                        .attributes(vec![("custom", "reds",)])
                        .contents(vec![HtmlNode::new_text("Centro comercial Moctezuma",)],),
                ),
                &HtmlNode::Tag(
                    HtmlTag::new("td")
                        .attributes(vec![("custom", "re-ds",)])
                        .contents(vec![HtmlNode::new_text("Francisco Chang",),]),
                ),
                &HtmlNode::Tag(
                    HtmlTag::new("td")
                        .attributes(vec![("custom", "something else",)])
                        .contents(vec![HtmlNode::new_text("Mexico"),]),
                ),
            ]
        );
        // Multi- Parent/Anscestor
        assert_eq!(
            doc.find(".tablebox > table td[custom]").nodes(),
            vec![
                &HtmlNode::Tag(
                    HtmlTag::new("td")
                        .attributes(vec![("custom", "reds",)])
                        .contents(vec![HtmlNode::new_text("Centro comercial Moctezuma",)],),
                ),
                &HtmlNode::Tag(
                    HtmlTag::new("td")
                        .attributes(vec![("custom", "re-ds",)])
                        .contents(vec![HtmlNode::new_text("Francisco Chang",),]),
                ),
                &HtmlNode::Tag(
                    HtmlTag::new("td")
                        .attributes(vec![("custom", "something else",)])
                        .contents(vec![HtmlNode::new_text("Mexico"),]),
                ),
            ]
        );
        // previous sibling
        assert_eq!(
            doc.find("td[custom=re-ds] ~ td").nodes(),
            vec![&HtmlNode::Tag(
                HtmlTag::new("td")
                    .attributes(vec![("custom", "something else",)])
                    .contents(vec![HtmlNode::new_text("Mexico"),]),
            ),]
        );
        // previous sibling once
        assert_eq!(
            doc.find("option[value=blue] + option").nodes(),
            vec![&HtmlNode::Tag(
                HtmlTag::new("option")
                    .attributes(vec![("value", "red"),])
                    .contents(vec![HtmlNode::new_text("Red")])
            ),]
        );
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

    /// Clears ther results list
    pub fn reset_result(&mut self) {
        self.results.clear();
    }

    /// Find all elements with the tag provided in the Html structure.
    pub fn find_str(&mut self, selector: &str) -> Result<&HtmlQuery, ParseHtmlError> {
        match CssSelector::from_str(selector) {
            Err(e) => Err(e),
            Ok(s) => Ok(self.find(&s)),
        }
    }

    /// Search through either the root HTML nodes if there are no results stored,
    /// otherwise search through the current results.
    pub fn find(&mut self, selector: &CssSelector) -> &HtmlQuery {
        if self.results.len() == 0 {
            self.find_from_root(selector);
        } else {
            self.find_from_results(selector);
        }
        self
    }

    fn find_from_root(&mut self, selector: &CssSelector) {
        let mut res = HtmlQueryResult {
            path: vec![(self.root, 0)],
        };
        // walk the tree and check for matches
        loop {
            if res.matches(selector) {
                self.results.push(res.clone());
            }
            match res.walk_next() {
                None => return,
                Some(_) => (),
            }
        }
    }

    fn find_from_results(&mut self, selector: &CssSelector) {
        let mut results = self.results.clone();
        self.results = vec![];
        for res in results {
            //result itself matches
            if res.matches(selector) {
                self.results.push(res.clone());
            }
            //check children of result only
            match res.get_node() {
                None => (),
                Some(n) => match n {
                    HtmlNode::Tag(t) => {
                        if t.contents.len() > 0 {
                            let mut new_res = HtmlQueryResult {
                                path: vec![(&t.contents, 0)],
                            };
                            // walk the tree below the current result and check for matches
                            loop {
                                if new_res.matches(selector) {
                                    //add the relative path to the path of the top level
                                    let mut add_res = res.clone();
                                    add_res.path.extend_from_slice(&new_res.path);
                                    self.results.push(add_res);
                                }
                                match new_res.walk_next() {
                                    None => return,
                                    Some(_) => (),
                                }
                            }
                        }
                    }
                    _ => (),
                },
            }
        }
    }

    pub fn nodes(&'a self) -> Vec<&'a HtmlNode> {
        let mut v = Vec::with_capacity(self.results.len());
        for r in &self.results {
            match r.get_node() {
                Some(node) => v.push(node),
                None => (),
            }
        }
        v
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
