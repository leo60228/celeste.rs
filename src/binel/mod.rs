use std::collections::HashMap;
use lazy_static::lazy_static;

/// `parser` parses `BinaryElement` files.
pub mod parser;

/// `writer` writes `BinaryElement` files.
pub mod writer;

/// `serialize` serializes and deserializes `BinEl`s.
pub mod serialize;

/// Holds `BinaryElement` files.
#[derive(Debug, PartialEq, Clone, Default)]
pub struct BinFile {
    pub package: String,
    pub root: BinEl,
}

/// A value stored in an attribute inside a `BinEl`. Unlike XML, attributes are strongly typed.
#[derive(Debug, PartialEq, Clone)]
pub enum BinElAttr {
    Bool(bool),
    Int(i32),
    Float(f32),
    Text(String)
}

/// An element stored in a `BinFile`. Based on XML.
#[derive(PartialEq, Debug, Clone, Default)]
pub struct BinEl {
    /// The name of the `BinEl`.
    pub name: String,
    /// All attributes of the `BinEl`. Unlike XML, these are strongly typed.
    pub attributes: HashMap<String, BinElAttr>,
    children: HashMap<String, Vec<BinEl>>
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
    pub fn children<'a>(&'a self) -> impl Iterator<Item=&Self> + 'a {
        self.children.values().flatten()
    }

    /// Get all children of the `BinEl`, mutable.
    #[inline]
    pub fn children_mut<'a>(&'a mut self) -> impl Iterator<Item=&mut Self> + 'a {
        self.children.values_mut().flatten()
    }

    /// Get children of the `BinEl` by name.
    #[inline]
    pub fn get(&self, name: &str) -> &Vec<Self> {
        self.children.get(name).unwrap_or(&CHILDLESS_BINEL_VEC)
    }

    /// Get mutable children of the `BinEl` by name.
    #[inline]
    pub fn get_mut(&mut self, name: &str) -> &mut Vec<Self> {
        self.children.entry(name.to_string()).or_insert_with(|| vec![])
    }

    /// Drain all children of the `BinEl`.
    #[inline]
    pub fn drain<'a>(&'a mut self) -> impl Iterator<Item=Self> + 'a {
        self.children.drain().map(|(_k, v)| v).flatten()
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
