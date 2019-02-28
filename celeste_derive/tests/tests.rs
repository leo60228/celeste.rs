use celeste_derive::BinElType;
use celeste::binel::{serialize::*, *};

#[derive(BinElType)]
struct EmptyMixedCase {}

#[derive(BinElType)]
struct OneField {
    number_field: i16
}

#[derive(BinElType)]
struct Recursive {
    elem_field: OneField,
    string_field: String
}

#[test]
fn create_empty() {
    let binel = match (EmptyMixedCase {}).as_binel() {
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

    let binel = match (OneField {number_field}).as_binel() {
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
fn create_recursive() {
    let number_field = -4;
    let string_field = "Hello, world!";

    let elem_field = OneField {number_field};
    let rec = Recursive {elem_field, string_field: string_field.to_string()};

    let binel = match rec.as_binel() {
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
