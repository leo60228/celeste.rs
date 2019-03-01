use celeste_derive::BinElType;
use celeste::binel::{serialize::*, *};

#[derive(Eq, PartialEq, Debug, BinElType)]
struct EmptyMixedCase {}

#[derive(Eq, PartialEq, Debug, BinElType, Clone)]
struct OneField {
    pub number_field: i16
}

#[derive(Eq, PartialEq, Debug, BinElType)]
struct Recursive {
    pub elem_field: OneField,
    pub string_field: String
}

#[derive(Eq, PartialEq, Debug, BinElType)]
#[celeste_name = "new/name"]
struct Renamed {
    #[celeste_name = "changed.field"]
    pub orig_name: u8,
    pub kept_name: u16
}

#[derive(Eq, PartialEq, Debug, BinElType)]
struct MultipleChildren {
    #[celeste_child_vec]
    pub children: Vec<OneField>,
    pub child: EmptyMixedCase
}

fn create_empty() -> BinEl {
    let binel = match (EmptyMixedCase {}).into_binel() {
        BinElValue::Element(elem) => elem,
        _ => panic!("Didn't get element!")
    };

    assert_eq!(binel.name, "emptyMixedCase");
    assert_eq!(binel.children().count(), 0);
    assert_eq!(binel.attributes.len(), 0);

    binel
}

#[test]
fn serialize_empty() {create_empty();}

#[test]
fn deserialize_empty() {
    let deserialized = EmptyMixedCase::from_binel(BinElValue::Element(create_empty()));
    assert_eq!(deserialized, Some(EmptyMixedCase {}));
}

fn create_attr() -> BinEl {
    let number_field = -4;

    let binel = match (OneField {number_field}).into_binel() {
        BinElValue::Element(elem) => elem,
        _ => panic!("Didn't get element!")
    };

    assert_eq!(binel.name, "oneField");
    assert_eq!(binel.children().count(), 0);
    assert_eq!(binel.attributes.len(), 1);

    match binel.attributes.get("numberField").unwrap() {
        BinElAttr::Int(num) => assert_eq!(*num, number_field as i32),
        _ => panic!("Didn't get int!")
    }

    binel
}

#[test]
fn serialize_attr() {create_attr();}

#[test]
fn deserialize_attr() {
    let deserialized = OneField::from_binel(BinElValue::Element(create_attr()));
    assert_eq!(deserialized, Some(OneField {number_field: -4}));
}

fn create_renamed() -> BinEl {
    let orig_name: u8 = 255;
    let kept_name: u16 = 65535;

    let binel = match (Renamed {orig_name, kept_name}).into_binel() {
        BinElValue::Element(elem) => elem,
        _ => panic!("Didn't get element!")
    };

    assert_eq!(binel.name, "new/name");
    assert_eq!(binel.children().count(), 0);
    assert_eq!(binel.attributes.len(), 2);

    match binel.attributes.get("changed.field").expect("orig_name wasn't renamed!") {
        BinElAttr::Int(num) => assert_eq!(*num, orig_name as i32),
        _ => panic!("Didn't get int!")
    }

    match binel.attributes.get("keptName").unwrap() {
        BinElAttr::Int(num) => assert_eq!(*num, kept_name as i32),
        _ => panic!("Didn't get int!")
    }

    binel
}

#[test]
fn deserialize_renamed() {
    let deserialized = Renamed::from_binel(BinElValue::Element(create_renamed()));
    assert_eq!(deserialized, Some(Renamed {orig_name: 255, kept_name: 65535}));
}

#[test]
fn serialize_renamed() {create_renamed();}

fn create_recursive() -> BinEl {
    let number_field = -4;
    let string_field = "Hello, world!";

    let elem_field = OneField {number_field};
    let rec = Recursive {elem_field, string_field: string_field.to_string()};

    let binel = match rec.into_binel() {
        BinElValue::Element(elem) => elem,
        _ => panic!("Didn't get element!")
    };

    assert_eq!(binel.name, "recursive");
    assert_eq!(binel.children().count(), 1);
    assert_eq!(binel.attributes.len(), 1);

    match binel.attributes.get("stringField").unwrap() {
        BinElAttr::Text(string) => assert_eq!(string, "Hello, world!"),
        _ => panic!("Didn't get text!")
    }

    let child = binel.children().next().unwrap();

    assert_eq!(child.name, "oneField");
    assert_eq!(child.children().count(), 0);
    assert_eq!(child.attributes.len(), 1);

    match child.attributes.get("numberField").unwrap() {
        BinElAttr::Int(num) => assert_eq!(*num, number_field as i32),
        _ => panic!("Didn't get int!")
    }

    binel
}

#[test]
fn deserialize_recursive() {
    let deserialized = Recursive::from_binel(BinElValue::Element(create_recursive()));
    assert_eq!(deserialized, Some(Recursive {
        string_field: "Hello, world!".to_string(),
        elem_field: OneField {
            number_field: -4
        }
    }));
}

#[test]
fn serialize_recursive() {create_recursive();}

fn create_child_vec() -> BinEl {
    let one_field = OneField { number_field: 5 };
    let vec = vec![one_field.clone(), one_field.clone(), one_field];

    let obj = MultipleChildren { children: vec, child: EmptyMixedCase {} };

    let binel = match obj.into_binel() {
        BinElValue::Element(elem) => elem,
        _ => panic!("Didn't get element!")
    };

    assert_eq!(binel.children().count(), 4);
    assert_eq!(binel.get("oneField").len(), 3);
    assert_eq!(binel.get("emptyMixedCase").len(), 1);

    for e in binel.get("oneField") {
        assert_eq!(e.children().count(), 0);
        assert_eq!(e.attributes.len(), 1);
        match e.attributes.get("numberField").unwrap() {
            BinElAttr::Int(num) => assert_eq!(*num, 5),
            _ => panic!("Didn't get int!")
        }
    }
    
    binel
}

#[test]
fn deserialize_child_vec() {
    let deserialized = MultipleChildren::from_binel(BinElValue::Element(create_child_vec()));
    assert_eq!(deserialized, Some(MultipleChildren {
        child: EmptyMixedCase {},
        children: vec![
            OneField { number_field: 5 },
            OneField { number_field: 5 },
            OneField { number_field: 5 }
        ]
    }));
}

#[test]
fn serialize_child_vec() {create_child_vec();}
