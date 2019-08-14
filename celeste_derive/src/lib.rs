#![recursion_limit = "2048"]

extern crate proc_macro;

use heck::MixedCase;
use syn::*;

mod data_struct;
mod newtype;

#[proc_macro_derive(BinElType, attributes(celeste_child_vec, celeste_name, celeste_skip))]
pub fn binel_type(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let ident = &input.ident;

    let mut name = ident.to_string().to_mixed_case();

    let is_newtype = match &input.data {
        Data::Struct(data) => {
            data.fields.iter().count() == 1 && data.fields.iter().all(|field| field.ident == None)
        }
        _ => false,
    };

    for ref attr in input.attrs.iter() {
        match attr.parse_meta() {
            Ok(Meta::Path(path)) => {
                let word = &path.segments.last().unwrap().ident;
                assert_ne!(
                    word.to_string(),
                    "celeste_name",
                    "celeste_name must have a value!"
                );
                assert_ne!(
                    word.to_string(),
                    "celeste_child_vec",
                    "celeste_child_vec is only valid on fields!"
                );
                assert_ne!(
                    word.to_string(),
                    "celeste_skip",
                    "celeste_skip is only valid on fields!"
                );
            }
            Ok(Meta::List(list)) => {
                assert_ne!(
                    list.path.segments.last().unwrap().ident.to_string(),
                    "celeste_name",
                    "celeste_name must have a value!"
                );
                assert_ne!(
                    list.path.segments.last().unwrap().ident.to_string(),
                    "celeste_child_vec",
                    "celeste_child_vec is only valid on fields!"
                );
                assert_ne!(
                    list.path.segments.last().unwrap().ident.to_string(),
                    "celeste_skip",
                    "celeste_skip is only valid on fields!"
                );
            }
            Ok(Meta::NameValue(kv)) => {
                if kv.path.segments.last().unwrap().ident.to_string() == "celeste_name" {
                    name = match kv.lit {
                        Lit::Str(string) => string.value(),
                        _ => panic!("celeste_name must be a string!"),
                    };
                }
                assert_ne!(
                    kv.path.segments.last().unwrap().ident.to_string(),
                    "celeste_child_vec",
                    "celeste_child_vec is only valid on fields!"
                );
                assert_ne!(
                    kv.path.segments.last().unwrap().ident.to_string(),
                    "celeste_skip",
                    "celeste_skip is only valid on fields!"
                );
            }
            _ => {}
        }
    }

    if is_newtype {
        newtype::binel_type_newtype(input, name)
    } else {
        data_struct::binel_type_struct(input, name)
    }
}
