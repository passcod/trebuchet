extern crate proc_macro;

use proc_macro2::{Ident, TokenStream};
use quote::{quote, quote_spanned};
use syn::{
    parse_macro_input, spanned::Spanned, Data, DeriveInput, Fields, FieldsNamed, FieldsUnnamed,
};

#[proc_macro_derive(FromValue)]
pub fn fromvalue_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = input.ident;

    let fvimpl = match input.data {
        Data::Struct(ref data) => match data.fields {
            Fields::Named(ref fields) => fvstruct(&name, fields),
            Fields::Unnamed(ref fields) => fvtuplestruct(&name, fields),
            _ => unimplemented!(),
        },
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

fn fvstruct(structname: &Ident, fields: &FieldsNamed) -> TokenStream {
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

fn fvtuplestruct(structname: &Ident, fields: &FieldsUnnamed) -> TokenStream {
    let len = fields.unnamed.len();

    let field_values = fields.unnamed.iter().map(|f| {
        let ty = &f.ty;
        quote_spanned! {f.span()=>
            <#ty as ::fromvalue::FromValue>::from(vec.remove(0))?,
        }
    });

    quote! {
        if let ::fromvalue::Value::Array(mut vec) = val {
            if vec.len() == #len {
                Ok(#structname(
                    #(#field_values)*
                ))
            } else {
                Err(stringify!(an array with #len items))
            }

        } else {
            Err("an array")
        }
    }
}
