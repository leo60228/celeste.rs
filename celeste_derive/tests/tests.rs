use celeste_derive::BinElType;
use celeste::binel::{serialize::*, *};

#[derive(BinElType)]
struct EmptyMixedCase {}

#[derive(BinElType, Clone)]
struct OneField {
    number_field: i16
}

#[derive(BinElType)]
struct Recursive {
    elem_field: OneField,
    string_field: String
}

#[derive(BinElType)]
#[celeste_name = "new/name"]
struct Renamed {
    #[celeste_name = "changed.field"]
    orig_name: u8,
    kept_name: f32
}

#[derive(BinElType)]
struct MultipleChildren {
    #[celeste_child_vec]
    children: Vec<OneField>,
    child: EmptyMixedCase
}

#[test]
fn create_empty() {
    let binel = match (EmptyMixedCase {}).into_binel() {
        BinElValue::Element(elem) => elem,
        _ => panic!("Didn't get element!")
    };

    assert_eq!(binel.name, "emptyMixedCase");
    assert_eq!(binel.children().count(), 0);
    assert_eq!(binel.attributes.len(), 0);
}

#[test]
fn create_attr() {
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
}

#[test]
fn create_renamed() {
    let int_field: u8 = 255;
    let float_field: f32 = 4.01;

    let binel = match (Renamed {orig_name: int_field, kept_name: float_field}).into_binel() {
        BinElValue::Element(elem) => elem,
        _ => panic!("Didn't get element!")
    };

    assert_eq!(binel.name, "new/name");
    assert_eq!(binel.children().count(), 0);
    assert_eq!(binel.attributes.len(), 2);

    match binel.attributes.get("changed.field").expect("orig_name wasn't renamed!") {
        BinElAttr::Int(num) => assert_eq!(*num, int_field as i32),
        _ => panic!("Didn't get int!")
    }

    match binel.attributes.get("keptName").unwrap() {
        BinElAttr::Float(num) => assert_eq!(*num, float_field),
        _ => panic!("Didn't get float!")
    }
}

#[test]
fn create_recursive() {
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
}

#[test]
fn create_child_vec() {
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
}
