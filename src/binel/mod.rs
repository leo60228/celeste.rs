use std::collections::HashMap;
use nom::lib::std::collections::hash_map::{Values, ValuesMut};

/// `parser` parses `BinaryElement` files.
pub mod parser;

/// `writer` writers `BinaryElement` files.
pub mod writer;

/// Holds `BinaryElement` files.
#[derive(Debug)]
pub struct BinFile {
    pub package: String,
    pub root: BinEl,
}

/// A value stored in an attribute inside a `BinEl`. Unlike XML, attributes are strongly typed.
#[derive(Debug, PartialEq)]
pub enum BinElAttr {
    Bool(bool),
    Int(i32),
    Float(f32),
    Text(String)
}

/// An element stored in a `BinFile`. Based on XML.
#[derive(PartialEq, Debug)]
pub struct BinEl {
    /// The name of the `BinEl`.
    pub name: String,
    children: HashMap<String, Vec<BinEl>>,
    /// All attributes of the `BinEl`. Unlike XML, these are strongly typed.
    pub attributes: HashMap<String, BinElAttr>
}

lazy_static! {
    static ref CHILDLESS_BINEL_VEC: Vec<BinEl> = vec![];
}

impl BinEl {
    /// Create a new `BinEl`.
    #[inline]
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            children: HashMap::new(),
            attributes: HashMap::new()
        }
    }

    /// Get the text content of the `BinEl`, if it exists.
    #[inline]
    pub fn text(&self) -> Option<&String> {
        match self.attributes.get("innerText")? {
            BinElAttr::Text(text) => Some(&text),
            _ => None
        }
    }

    /// Get the mutable text content of the `BinEl`, if it exists.
    #[inline]
    pub fn text_mut(&mut self) -> Option<&mut String> {
        match self.attributes.get_mut("innerText")? {
            BinElAttr::Text(ref mut text) => Some(text),
            _ => None
        }
    }

    /// Set the text content of the `BinEl`.
    #[inline]
    pub fn set_text(&mut self, text: &str) -> Option<BinElAttr> {
        self.attributes.insert("innerText".to_string(), BinElAttr::Text(text.to_string()))
    }

    /// Add a child to the `BinEl`.
    #[inline]
    pub fn insert(&mut self, child: Self) {
        self.get_mut(&child.name).push(child);
    }

    /// Get all children of the `BinEl`.
    #[inline]
    pub fn children(&self) -> Values<String, Vec<Self>> {
        self.children.values()
    }

    /// Get all children of the `BinEl`, mutable.
    #[inline]
    pub fn children_mut(&mut self) -> ValuesMut<String, Vec<Self>> {
        self.children.values_mut()
    }

    /// Get children of the `BinEl` by name.
    #[inline]
    pub fn get(&self, name: &str) -> &Vec<Self> {
        if !self.children.contains_key(name) {
          return &CHILDLESS_BINEL_VEC;
        }

        self.children.get(name).expect("should always exist")
    }

    /// Get mutable children of the `BinEl` by name.
    #[inline]
    pub fn get_mut(&mut self, name: &str) -> &mut Vec<Self> {
        if !self.children.contains_key(name) {
            self.children.insert(name.to_string(), vec![]);
        }

        self.children.get_mut(name).expect("should always exist")
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn insert_child() {
        let mut file = BinFile {
            package: "pkg".to_string(),
            root: BinEl::new("root")
        };
        file.root.insert(BinEl::new("one"));
        file.root.insert(BinEl::new("two"));
    }

    #[test]
    fn get_child() {
        let empty_binel: Vec<BinEl> = vec![];
        let mut file = BinFile {
            package: "pkg".to_string(),
            root: BinEl::new("root")
        };
        file.root.insert(BinEl::new("one"));
        file.root.insert(BinEl::new("two"));
        assert_eq!(file.root.get_mut("one")[0].set_text("hello"), None);
        assert_eq!(file.root.get_mut("two")[0].attributes.insert("word".to_string(), BinElAttr::Text("world".to_string())), None);

        assert_eq!(file.root.get("one")[0].text(), Some(&"hello".to_string()));
        assert_eq!(file.root.get("two")[0].text(), None);
        assert_eq!(file.root.get("two")[0].attributes.get("word"), Some(&BinElAttr::Text("world".to_string())));
        assert_eq!(file.root.get("three"), &empty_binel);
    }
}
