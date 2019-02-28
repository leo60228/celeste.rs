#![recursion_limit="2048"]

extern crate proc_macro;

use quote::quote;
use syn::*;
use heck::MixedCase;

#[proc_macro_derive(BinElType, attributes(celeste_multiple_children, celeste_name))]
pub fn binel_type(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let ident = &input.ident;

    let mut name = ident.to_string().to_mixed_case();

    for ref attr in input.attrs.iter() {
        match attr.parse_meta() {
            Ok(Meta::Word(word)) => {
                assert_ne!(word.to_string(), "celeste_name", "celeste_name must have a value!");
                assert_ne!(word.to_string(), "celeste_multiple_children", "celeste_multiple_children is only valid on fields!");
            },
            Ok(Meta::List(list)) => {
                assert_ne!(list.ident.to_string(), "celeste_name", "celeste_name must have a value!");
                assert_ne!(list.ident.to_string(), "celeste_multiple_children", "celeste_multiple_children is only valid on fields!");
            },
            Ok(Meta::NameValue(kv)) => {
                if kv.ident.to_string() == "celeste_name" {
                    name = match kv.lit {
                        Lit::Str(string) => string.value(),
                        _ => panic!("celeste_name must be a string!")
                    };
                }
                assert_ne!(kv.ident.to_string(), "celeste_multiple_children", "celeste_multiple_children is only valid on fields!");
            },
            _ => {}
        }
    }

    let body = match input.data {
        Data::Struct(body) => body,
        _ => panic!("You can only derive BinElType on structs!")
    };

    let mut idents = Vec::new();
    let mut names = Vec::new();

    let mut iter_idents = Vec::new();
    let mut iter_names = Vec::new();

    for ref field in body.fields.iter() {
        let mut is_iter = false;
        let ident = match &field.ident {
            &Some(ref ident) => ident.clone(),
            &None => panic!("Your struct is missing a field identity!"),
        };
        let mut name = ident.to_string().to_mixed_case();
        for ref attr in field.attrs.iter() {
            match attr.parse_meta() {
                Ok(Meta::Word(word)) => {
                    if word.to_string() == "celeste_multiple_children" {
                        is_iter = true;
                    }
                    assert_ne!(word.to_string(), "celeste_name", "celeste_name must have a value!");
                },
                Ok(Meta::List(list)) => {
                    assert_ne!(list.ident.to_string(), "celeste_name", "celeste_name must have a value!");
                    assert_ne!(list.ident.to_string(), "celeste_multiple_children", "celeste_multiple_children has no arguments!");
                },
                Ok(Meta::NameValue(kv)) => {
                    if kv.ident.to_string() == "celeste_name" {
                        name = match kv.lit {
                            Lit::Str(string) => string.value(),
                            _ => panic!("celeste_name must be a string!")
                        };
                    }
                    assert_ne!(kv.ident.to_string(), "celeste_multiple_children", "celeste_multiple_children has no arguments!");
                },
                _ => {}
            }
        }
        if !is_iter {
            idents.push(ident);
            names.push(name);
        } else {
            iter_idents.push(ident);
            iter_names.push(name);
        }
    }

    // disable mutability
    let idents = idents;
    let names = names;
    let iter_idents = iter_idents;
    let iter_names = iter_names;

    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let output = quote! {
        impl #impl_generics ::celeste::binel::serialize::BinElType for #ident #ty_generics #where_clause {
            fn from_binel(binel: ::celeste::binel::serialize::BinElValue) -> Option<Self> {
                use ::celeste::binel::*;

                None
            }

            fn as_binel(self) -> ::celeste::binel::serialize::BinElValue {
                use ::celeste::binel::*;

                let mut binel = BinEl::new(#name);

                #(
                    match serialize::BinElType::as_binel(self.#idents) {
                        serialize::BinElValue::Attribute(attr) => {
                            binel.attributes.insert(#names.to_string(), attr);
                        },
                        serialize::BinElValue::Element(child) => {
                            binel.insert(child);
                        }
                    };
                )*

                serialize::BinElValue::Element(binel)
            }
        }
    };

    proc_macro::TokenStream::from(output)
}
