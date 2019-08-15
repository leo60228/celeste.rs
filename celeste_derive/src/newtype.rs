use quote::quote;
use syn::*;

pub(crate) fn binel_type_newtype(input: DeriveInput, name: String) -> proc_macro::TokenStream {
    let ident = input.ident;

    let check_name = name.clone();
    let err_name = name.clone();

    let ty = match &input.data {
        Data::Struct(data) => &data.fields.iter().next().unwrap().ty,
        _ => panic!("Newtype derive implementation received non-struct input!"),
    };

    let attr_ty = ty.clone();
    let elem_ty = ty.clone();

    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let output = quote! {
        impl #impl_generics ::celeste::binel::serialize::BinElType for #ident #ty_generics #where_clause {
            fn from_binel(binel: ::celeste::binel::serialize::BinElValue) -> ::celeste::Result<'static, Self> {
                use ::celeste::binel::*;
                use ::celeste::Error;

                if let serialize::BinElValue::Element(elem) = &binel {
                    if elem.name != #check_name {
                        return Err(Error::wrong_name(#err_name, elem.name.clone()));
                    }
                }

                match <#ty as serialize::BinElType>::from_binel(binel) {
                    Ok(val) => Ok(Self(val)),
                    Err(err) => Err(err)
                }
            }

            fn into_binel(self) -> ::celeste::binel::serialize::BinElValue {
                use ::celeste::binel::*;

                serialize::BinElType::into_binel(self.0)
            }

            fn maybe_attr() -> bool { <#attr_ty as ::celeste::binel::serialize::BinElType>::maybe_attr() }
            fn maybe_elem() -> bool { <#elem_ty as ::celeste::binel::serialize::BinElType>::maybe_elem() }

            fn elem_name() -> Option<&'static str> { Some(#name) }
        }
    };

    proc_macro::TokenStream::from(output)
}
