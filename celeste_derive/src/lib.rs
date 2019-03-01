#![recursion_limit="2048"]

extern crate proc_macro;

use quote::quote;
use syn::*;
use heck::MixedCase;

#[proc_macro_derive(BinElType, attributes(celeste_child_vec, celeste_name))]
pub fn binel_type(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let ident = &input.ident;

    let mut name = ident.to_string().to_mixed_case();

    for ref attr in input.attrs.iter() {
        match attr.parse_meta() {
            Ok(Meta::Word(word)) => {
                assert_ne!(word.to_string(), "celeste_name", "celeste_name must have a value!");
                assert_ne!(word.to_string(), "celeste_child_vec", "celeste_child_vec is only valid on fields!");
            },
            Ok(Meta::List(list)) => {
                assert_ne!(list.ident.to_string(), "celeste_name", "celeste_name must have a value!");
                assert_ne!(list.ident.to_string(), "celeste_child_vec", "celeste_child_vec is only valid on fields!");
            },
            Ok(Meta::NameValue(kv)) => {
                if kv.ident.to_string() == "celeste_name" {
                    name = match kv.lit {
                        Lit::Str(string) => string.value(),
                        _ => panic!("celeste_name must be a string!")
                    };
                }
                assert_ne!(kv.ident.to_string(), "celeste_child_vec", "celeste_child_vec is only valid on fields!");
            },
            _ => {}
        }
    }

    let body = match input.data {
        Data::Struct(body) => body,
        _ => panic!("You can only derive BinElType on structs!")
    };

    let mut s_idents = Vec::new();
    let mut s_names = Vec::new();
    let mut s_vec_idents = Vec::new();

    let mut d_types = Vec::new();
    let mut d_vec_types = Vec::new();
    let mut d_vec_types_inner = Vec::new();
    let mut d_vec_names = Vec::new();

    for ref field in body.fields.iter() {
        let mut is_vec = false;
        let ident = match &field.ident {
            &Some(ref ident) => ident.clone(),
            &None => panic!("Your struct is missing a field identity!"),
        };
        let mut name = ident.to_string().to_mixed_case();
        for ref attr in field.attrs.iter() {
            match attr.parse_meta() {
                Ok(Meta::Word(word)) => {
                    if word.to_string() == "celeste_child_vec" {
                        is_vec = true;
                    }
                    assert_ne!(word.to_string(), "celeste_name", "celeste_name must have a value!");
                },
                Ok(Meta::List(list)) => {
                    assert_ne!(list.ident.to_string(), "celeste_name", "celeste_name must have a value!");
                    assert_ne!(list.ident.to_string(), "celeste_child_vec", "celeste_child_vec has no arguments!");
                },
                Ok(Meta::NameValue(kv)) => {
                    if kv.ident.to_string() == "celeste_name" {
                        name = match kv.lit {
                            Lit::Str(string) => string.value(),
                            _ => panic!("celeste_name must be a string!")
                        };
                    }
                    assert_ne!(kv.ident.to_string(), "celeste_child_vec", "celeste_child_vec has no arguments!");
                },
                _ => {}
            }
        }
        if !is_vec {
            s_idents.push(ident);
            s_names.push(name);
            d_types.push(field.ty.clone());
        } else {
            s_vec_idents.push(ident);

            d_vec_types.push(field.ty.clone());
            
            match &field.ty {
                Type::Path(path) => {
                    let path = &path.path;

                    let last = path.segments.last();

                    let last = match last {
                        Some(punctuated::Pair::Punctuated(segment, _)) => segment,
                        Some(punctuated::Pair::End(segment)) => segment,
                        _ => panic!("A field with celeste_child_vec's type must be generic!")
                    };

                    match &last.arguments {
                        PathArguments::AngleBracketed(args) => {
                            let args = &args.args;

                            let mut found_type = false;

                            for e in args {
                                if let GenericArgument::Type(typ) = e {
                                    if found_type {
                                        panic!("A field with celeste_child_vec's type must have a single generic argument!");
                                    }

                                    d_vec_types_inner.push(typ.clone());
                                    found_type = true;
                                }
                            }

                            if !found_type {
                                panic!("A field with celeste_child_vec's type must have a single generic argument!");
                            }
                        },
                        _ => panic!("A field with celeste_child_vec's type must be generic!")
                    }
                },
                _ => panic!("A field with celeste_child_vec's type must be a normal type!")
            }

            d_vec_names.push(name);
        }
    }

    // disable mutability and make into iterators
    let d_idents = s_idents.iter();
    let d_names = s_names.iter();
    let d_types = d_types.iter();
    let d_vec_idents = s_vec_idents.iter();
    let d_vec_types_option = d_vec_types_inner.iter();
    let d_vec_types_inner = d_vec_types_inner.iter();
    let d_fields = s_idents.iter().chain(s_vec_idents.iter());
    let s_idents = s_idents.iter();
    let s_names = s_names.iter();
    let s_vec_idents = s_vec_idents.iter();

    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let output = quote! {
        impl #impl_generics ::celeste::binel::serialize::BinElType for #ident #ty_generics #where_clause {
            fn from_binel(mut binel: ::celeste::binel::serialize::BinElValue) -> Option<Self> {
                use ::celeste::binel::*;

                let mut binel = match binel {
                    serialize::BinElValue::Element(elem) => elem,
                    _ => return None
                };

                #(
                    let name = #d_names;

                    let attr = binel.attributes.remove(name); // move out of map
                    let child = binel.get_mut(name); // move out of vec
                    
                    let field = match (child.len(), attr) {
                        (1, Some(_)) => return None, // having both is invalid
                        (0, None) => return None, // having neither is invalid
                        (1, None) => serialize::BinElValue::Element(child.pop().unwrap()), // single child
                        (0, Some(attr)) => serialize::BinElValue::Attribute(attr), // attribute
                        _ => return None // more than one child
                    };
                    
                    let #d_idents = <#d_types as serialize::BinElType>::from_binel(field)?;
                )*

                #(
                    let children = binel.get_mut(#d_vec_names); // move out of vec
                    
                    let mut maybe_valid: Vec<Option<#d_vec_types_option>> = 
                        children.drain(..)
                                .map(|e| <#d_vec_types_inner as serialize::BinElType>
                                         ::from_binel(serialize::BinElValue::Element(e)))
                                .collect();

                    if maybe_valid.iter().any(|e| e.is_none()) {
                        return None;
                    }

                    let #d_vec_idents = maybe_valid.drain(..).map(|e| e.unwrap()).collect();
                )*

                let new: Self = Self { #(#d_fields),* };

                Some(new)
            }

            fn into_binel(self) -> ::celeste::binel::serialize::BinElValue {
                use ::celeste::binel::*;

                let mut binel = BinEl::new(#name);

                #(
                    match serialize::BinElType::into_binel(self.#s_idents) {
                        serialize::BinElValue::Attribute(attr) => {
                            binel.attributes.insert(#s_names.to_string(), attr);
                        },
                        serialize::BinElValue::Element(child) => {
                            binel.insert(child);
                        }
                    };
                )*

                #(
                    for child in self.#s_vec_idents {
                        match serialize::BinElType::into_binel(child) {
                            serialize::BinElValue::Attribute(_) => panic!("Can't serialize Vec of attributes!"),
                            serialize::BinElValue::Element(child) => {
                                binel.insert(child);
                            }
                        }
                    }
                )*

                serialize::BinElValue::Element(binel)
            }
        }
    };

    proc_macro::TokenStream::from(output)
}
