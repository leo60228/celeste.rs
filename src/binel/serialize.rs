use std::prelude::v1::*;
use super::*;
use crate::{Error, Result};

#[cfg(feature = "celeste_derive")]
pub use celeste_derive::*;

/// A value for serializing and deserializing to and from `BinEl`s.
pub enum BinElValue {
    Attribute(BinElAttr),
    Element(BinEl),
    None,
}

/// A type that can be serialized to and from a `BinEl`.
///
/// # Deriving
/// This type can be derived through a proc macro if the `celeste_derive` feature is enabled. This
/// is the default. If the type is a normal struct, then it will be serialized as an element, named
/// using the `celeste_name` key-value attribute (ex. `#[celeste_name = "element"]`), or defaulting
/// to the name of the struct in mixed case (converted via `heck`). All fields that serialize to a
/// `BinElValue::Element` will get serialized as a child, and ones that serialize to a
/// `BinElValue::Attribute` get added as an attribute. The attribute is named using `celeste_name`,
/// again defaulting to mixed case.
///
/// Alternatively, if it is a newtype (i.e. tuple struct with single type), then it will get
/// serialized as the type it is based off of. If this is an element, it must have the same name as
/// the newtype struct (`celeste_name` is allowed). This can be useful for stubbing out complex
/// elements that you still want to include in the output.
pub trait BinElType: Sized {
    /// Serialize self into a BinElValue. Can never fail.
    fn into_binel(self) -> BinElValue;

    /// Deserialize self from a BinElValue.
    fn from_binel(binel: BinElValue) -> Result<'static, Self>;

    /// Whether the `BinElType` may serialize to a `BinElValue::Attribute`. Recommended.
    fn maybe_attr() -> bool {
        true
    }

    /// Whether the `BinElType` may serialize to a `BinElValue::Element`. Recommended.
    fn maybe_elem() -> bool {
        true
    }

    /// If there is a single valid name when deserializing from an element, and you know it at
    /// compile-time, implement this. Recommended.
    fn elem_name() -> Option<&'static str> {
        None
    }
}

macro_rules! impl_primitive {
    ($attr:ident, $type:ident, $val:ident) => {
        impl BinElType for $type {
            fn into_binel(self) -> BinElValue {
                BinElValue::Attribute(BinElAttr::$attr($val::from(self)))
            }

            #[allow(clippy::cast_lossless)]
            fn from_binel(binel: BinElValue) -> Result<'static, Self> {
                match binel {
                    BinElValue::Attribute(BinElAttr::$attr(e)) => Ok(e as $type),
                    _ => Err(Error::from_name(stringify!($type))),
                }
            }

            fn maybe_elem() -> bool {
                false
            }
        }
    };
}

impl_primitive!(Bool, bool, bool);
impl_primitive!(Int, u8, i32);
impl_primitive!(Int, u16, i32);
impl_primitive!(Int, i8, i32);
impl_primitive!(Int, i16, i32);
impl_primitive!(Int, i32, i32);
impl_primitive!(Float, f32, f32);
impl_primitive!(Text, String, String);

impl BinElType for BinEl {
    fn into_binel(self) -> BinElValue {
        BinElValue::Element(self)
    }

    fn from_binel(binel: BinElValue) -> Result<'static, Self> {
        match binel {
            BinElValue::Element(e) => Ok(e),
            _ => Err(Error::from_name("BinElValue")),
        }
    }

    fn maybe_attr() -> bool {
        false
    }
}

impl BinElType for BinElAttr {
    fn into_binel(self) -> BinElValue {
        BinElValue::Attribute(self)
    }

    fn from_binel(binel: BinElValue) -> Result<'static, Self> {
        match binel {
            BinElValue::Attribute(e) => Ok(e),
            _ => Err(Error::from_name("BinElAttr")),
        }
    }

    fn maybe_elem() -> bool {
        false
    }
}

impl<T: BinElType> BinElType for Option<T> {
    fn into_binel(self) -> BinElValue {
        match self {
            Some(inner) => inner.into_binel(),
            None => BinElValue::None,
        }
    }

    fn from_binel(binel: BinElValue) -> Result<'static, Self> {
        Ok(T::from_binel(binel).ok())
    }
}

#[cfg(all(test, feature = "std"))]
mod test {
    use crate::binel::serialize::{BinElType, BinElValue};
    use crate::binel::{BinEl, BinElAttr};

    #[derive(Eq, PartialEq, Debug, BinElType)]
    struct EmptyMixedCase {}

    #[derive(Eq, PartialEq, Debug, BinElType, Clone)]
    struct OneField {
        pub number_field: i16,
    }

    #[derive(Eq, PartialEq, Debug, BinElType)]
    struct Recursive {
        pub elem_field: OneField,
        pub string_field: String,
    }

    #[derive(Eq, PartialEq, Debug, BinElType)]
    #[celeste_name = "new/name"]
    struct Renamed {
        #[celeste_name = "changed.field"]
        pub orig_name: u8,
        pub kept_name: u16,
    }

    #[derive(Eq, PartialEq, Debug, BinElType)]
    struct MultipleChildren {
        #[celeste_child_vec]
        pub children: Vec<OneField>,
        pub child: EmptyMixedCase,
    }

    #[derive(Eq, PartialEq, Debug, BinElType)]
    struct Optional {
        pub child: Option<EmptyMixedCase>,
    }

    #[derive(Eq, PartialEq, Debug, BinElType)]
    struct Skip {
        #[celeste_skip]
        pub skipped: String,
        pub kept: String,
    }

    #[derive(PartialEq, Debug, Clone, BinElType)]
    struct Newtype(pub BinEl);

    fn create_empty() -> BinEl {
        let binel = match (EmptyMixedCase {}).into_binel() {
            BinElValue::Element(elem) => elem,
            _ => panic!("Didn't get element!"),
        };

        assert_eq!(binel.name, "emptyMixedCase");
        assert_eq!(binel.children().count(), 0);
        assert_eq!(binel.attributes.len(), 0);

        binel
    }

    #[test]
    fn serialize_empty() {
        create_empty();
    }

    #[test]
    fn deserialize_empty() {
        let deserialized = EmptyMixedCase::from_binel(BinElValue::Element(create_empty()));
        assert_eq!(deserialized.unwrap(), EmptyMixedCase {});
    }

    fn create_attr() -> BinEl {
        let number_field = -4;

        let binel = match (OneField { number_field }).into_binel() {
            BinElValue::Element(elem) => elem,
            _ => panic!("Didn't get element!"),
        };

        assert_eq!(binel.name, "oneField");
        assert_eq!(binel.children().count(), 0);
        assert_eq!(binel.attributes.len(), 1);

        match binel.attributes.get("numberField").unwrap() {
            BinElAttr::Int(num) => assert_eq!(*num, number_field as i32),
            _ => panic!("Didn't get int!"),
        }

        binel
    }

    #[test]
    fn serialize_attr() {
        create_attr();
    }

    #[test]
    fn deserialize_attr() {
        let deserialized = OneField::from_binel(BinElValue::Element(create_attr()));
        assert_eq!(deserialized.unwrap(), OneField { number_field: -4 });
    }

    fn create_renamed() -> BinEl {
        let orig_name: u8 = 255;
        let kept_name: u16 = 65535;

        let binel = match (Renamed {
            orig_name,
            kept_name,
        })
        .into_binel()
        {
            BinElValue::Element(elem) => elem,
            _ => panic!("Didn't get element!"),
        };

        assert_eq!(binel.name, "new/name");
        assert_eq!(binel.children().count(), 0);
        assert_eq!(binel.attributes.len(), 2);

        match binel
            .attributes
            .get("changed.field")
            .expect("orig_name wasn't renamed!")
        {
            BinElAttr::Int(num) => assert_eq!(*num, orig_name as i32),
            _ => panic!("Didn't get int!"),
        }

        match binel.attributes.get("keptName").unwrap() {
            BinElAttr::Int(num) => assert_eq!(*num, kept_name as i32),
            _ => panic!("Didn't get int!"),
        }

        binel
    }

    #[test]
    fn deserialize_renamed() {
        let deserialized = Renamed::from_binel(BinElValue::Element(create_renamed()));
        assert_eq!(
            deserialized.unwrap(),
            Renamed {
                orig_name: 255,
                kept_name: 65535
            }
        );
    }

    #[test]
    fn serialize_renamed() {
        create_renamed();
    }

    fn create_recursive() -> BinEl {
        let number_field = -4;
        let string_field = "Hello, world!";

        let elem_field = OneField { number_field };
        let rec = Recursive {
            elem_field,
            string_field: string_field.to_string(),
        };

        let binel = match rec.into_binel() {
            BinElValue::Element(elem) => elem,
            _ => panic!("Didn't get element!"),
        };

        assert_eq!(binel.name, "recursive");
        assert_eq!(binel.children().count(), 1);
        assert_eq!(binel.attributes.len(), 1);

        match binel.attributes.get("stringField").unwrap() {
            BinElAttr::Text(string) => assert_eq!(string, "Hello, world!"),
            _ => panic!("Didn't get text!"),
        }

        let child = binel.children().next().unwrap();

        assert_eq!(child.name, "oneField");
        assert_eq!(child.children().count(), 0);
        assert_eq!(child.attributes.len(), 1);

        match child.attributes.get("numberField").unwrap() {
            BinElAttr::Int(num) => assert_eq!(*num, number_field as i32),
            _ => panic!("Didn't get int!"),
        }

        binel
    }

    #[test]
    fn deserialize_recursive() {
        let deserialized = Recursive::from_binel(BinElValue::Element(create_recursive()));
        assert_eq!(
            deserialized.unwrap(),
            Recursive {
                string_field: "Hello, world!".to_string(),
                elem_field: OneField { number_field: -4 }
            }
        );
    }

    #[test]
    fn serialize_recursive() {
        create_recursive();
    }

    fn create_child_vec() -> BinEl {
        let one_field = OneField { number_field: 5 };
        let vec = vec![one_field.clone(), one_field.clone(), one_field];

        let obj = MultipleChildren {
            children: vec,
            child: EmptyMixedCase {},
        };

        let binel = match obj.into_binel() {
            BinElValue::Element(elem) => elem,
            _ => panic!("Didn't get element!"),
        };

        assert_eq!(binel.children().count(), 4);
        assert_eq!(binel.get("oneField").len(), 3);
        assert_eq!(binel.get("emptyMixedCase").len(), 1);

        for e in binel.get("oneField") {
            assert_eq!(e.children().count(), 0);
            assert_eq!(e.attributes.len(), 1);
            match e.attributes.get("numberField").unwrap() {
                BinElAttr::Int(num) => assert_eq!(*num, 5),
                _ => panic!("Didn't get int!"),
            }
        }

        binel
    }

    #[test]
    fn deserialize_child_vec() {
        let deserialized = MultipleChildren::from_binel(BinElValue::Element(create_child_vec()));
        assert_eq!(
            deserialized.unwrap(),
            MultipleChildren {
                child: EmptyMixedCase {},
                children: vec![
                    OneField { number_field: 5 },
                    OneField { number_field: 5 },
                    OneField { number_field: 5 }
                ]
            }
        );
    }

    #[test]
    fn serialize_child_vec() {
        create_child_vec();
    }

    fn create_optional_some() -> BinEl {
        let binel = match (Optional {
            child: Some(EmptyMixedCase {}),
        })
        .into_binel()
        {
            BinElValue::Element(elem) => elem,
            _ => panic!("Didn't get element!"),
        };

        assert_eq!(binel.children().count(), 1);
        assert_eq!(binel.get("emptyMixedCase").len(), 1);

        binel
    }

    #[test]
    fn serialize_optional_some() {
        create_optional_some();
    }

    #[test]
    fn deserialize_optional_some() {
        assert_eq!(
            Optional::from_binel(BinElValue::Element(create_optional_some())).unwrap(),
            Optional {
                child: Some(EmptyMixedCase {})
            }
        );
    }

    fn create_optional_none() -> BinEl {
        let binel = match (Optional { child: None }).into_binel() {
            BinElValue::Element(elem) => elem,
            _ => panic!("Didn't get element!"),
        };

        assert_eq!(binel.children().count(), 0);

        binel
    }

    #[test]
    fn serialize_optional_none() {
        create_optional_none();
    }

    #[test]
    fn deserialize_optional_none() {
        assert_eq!(
            Optional::from_binel(BinElValue::Element(create_optional_none())).unwrap(),
            Optional { child: None }
        );
    }

    fn create_skip() -> BinEl {
        let binel = match (Skip {
            skipped: "hi".to_string(),
            kept: "bye".to_string(),
        })
        .into_binel()
        {
            BinElValue::Element(elem) => elem,
            _ => panic!("Didn't get element!"),
        };

        assert_eq!(binel.children().count(), 0);
        assert_eq!(binel.attributes.len(), 1);
        assert_eq!(
            binel.attributes.get("kept").unwrap(),
            &BinElAttr::Text("bye".to_string())
        );

        binel
    }

    #[test]
    fn serialize_skip() {
        create_skip();
    }

    #[test]
    fn deserialize_skip() {
        let mut binel = create_skip();

        binel
            .attributes
            .insert("skipped".to_string(), BinElAttr::Text("oh no!".to_string()));

        assert_eq!(
            Skip::from_binel(BinElValue::Element(binel)).unwrap(),
            Skip {
                skipped: Default::default(),
                kept: "bye".to_string()
            }
        );
    }

    fn create_newtype() -> (BinEl, Newtype) {
        let mut binel = BinEl::new("newtype");
        binel.insert(create_recursive());
        binel
            .attributes
            .insert("test".to_string(), BinElAttr::Int(5));
        let newtype = Newtype(binel.clone());
        let elem = match newtype.clone().into_binel() {
            BinElValue::Element(elem) => elem,
            _ => panic!("Didn't get element!"),
        };
        assert_eq!(elem, binel);
        assert_eq!(elem, newtype.0);
        (binel, newtype)
    }

    #[test]
    fn serialize_newtype() {
        create_newtype();
    }

    #[test]
    fn deserialize_newtype() {
        let (binel, newtype) = create_newtype();
        let deserialized = Newtype::from_binel(BinElValue::Element(binel));
        assert_eq!(deserialized.unwrap(), newtype);
    }
}
