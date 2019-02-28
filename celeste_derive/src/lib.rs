#![recursion_limit="2048"]

extern crate proc_macro;

use quote::quote;
use syn::*;
use heck::MixedCase;

#[proc_macro_derive(BinElType)]
pub fn binel_type(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let body = match input.data {
        Data::Struct(body) => body,
        _ => panic!("You can only derive BinElType on structs!")
    };

    let mut idents = Vec::new();
    for ref field in body.fields.iter() {
        match &field.ident {
            &Some(ref ident) => idents.push(ident.clone()),
            &None => panic!("Your struct is missing a field identity!"),
        }
    }
    let idents = idents; // disable mutability

    let name = &input.ident;

    let mixed_name = name.to_string().to_mixed_case();
    let mixed_idents: Vec<String> = idents.iter().map(|e| e.to_string().to_mixed_case()).collect();

    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let output = quote! {
        impl #impl_generics ::celeste::binel::serialize::BinElType for #name #ty_generics #where_clause {
            fn from_binel(binel: ::celeste::binel::serialize::BinElValue) -> Option<Self> {
                use ::celeste::binel::*;

                None
            }

            fn as_binel(self) -> ::celeste::binel::serialize::BinElValue {
                use ::celeste::binel::*;

                let mut binel = BinEl::new(#mixed_name);

                #(
                    match serialize::BinElType::as_binel(self.#idents) {
                        serialize::BinElValue::Attribute(attr) => {
                            binel.attributes.insert(#mixed_idents.to_string(), attr);
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
