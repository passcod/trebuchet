extern crate proc_macro;

use proc_macro2::{Ident, TokenStream};
use quote::{quote, quote_spanned};
use syn::{parse_macro_input, spanned::Spanned, Data, DataStruct, DeriveInput, Fields};

#[proc_macro_derive(FromValue)]
pub fn fromvalue_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = input.ident;

    let fvimpl = match input.data {
        Data::Struct(ref data) => fvstruct(&name, data),
        _ => unimplemented!(),
    };

    let expanded = quote! {
        impl ::fromvalue::FromValue for #name {
            fn from(val: ::fromvalue::Value) -> Result<Self, &'static str> {
                #fvimpl
            }
        }
    };

    proc_macro::TokenStream::from(expanded)
}

fn fvstruct(structname: &Ident, data: &DataStruct) -> TokenStream {
    match data.fields {
        Fields::Named(ref fields) => {
            let key_checks = fields.named.iter().map(|f| {
                let name = &f.ident;
                quote_spanned! {f.span()=>
                    if !map.contains_key(stringify!(#name)) {
                        return Err(stringify!(the #name field));
                    }
                }
            });

            let field_values = fields.named.iter().map(|f| {
                let name = &f.ident;
                let ty = &f.ty;
                quote_spanned! {f.span()=>
                    let #name = <#ty as ::fromvalue::FromValue>::from(map.remove(stringify!(#name)).unwrap())?;
                }
            });

            let field_sets = fields.named.iter().map(|f| {
                let name = &f.ident;
                quote_spanned! {f.span()=> #name, }
            });

            quote! {
                if let ::fromvalue::Value::Object(mut map) = val {
                    #(#key_checks)*
                    #(#field_values)*

                    Ok(#structname {
                        #(#field_sets)*
                    })
                } else {
                    Err("an object")
                }
            }
        }
        _ => unimplemented!(),
    }
}
