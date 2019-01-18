extern crate proc_macro as pm1;

use proc_macro2::{Ident, Literal, Punct, Spacing, TokenStream, TokenTree};
use quote::{quote, quote_spanned, ToTokens};
use syn::spanned::Spanned;
use syn::{parse_macro_input, parse_quote, FnArg, ImplItem, ItemImpl, Type};

#[proc_macro]
pub fn rpc_impl_struct(input: pm1::TokenStream) -> pm1::TokenStream {
    let mut in_impl = parse_macro_input!(input as ItemImpl);

    // extract methods
    let methods = in_impl.items.iter().filter_map(|item| match item {
        ImplItem::Method(m) => Some(m),
        _ => None,
    });

    // generate delegated entries for methods we care about
    let delegates = methods.map(|method| {
        let name = &method.sig.ident;
        let output = &method.sig.decl.output;
        let inputs = &method.sig.decl.inputs;

        let types: Vec<&Type> = inputs
            .iter()
            .filter_map(|input| {
                if let FnArg::Captured(arg) = input {
                    Some(&arg.ty)
                } else {
                    None
                }
            })
            .collect();

        let args = (0..types.len()).map(|n| {
            let mut stream = TokenStream::new();
            let tree: Vec<TokenTree> = vec![
                Ident::new("args", method.span()).into(),
                Punct::new('.', Spacing::Alone).into(),
                Literal::usize_unsuffixed(n).into(),
            ];
            stream.extend(tree);
            stream
        });

        let typdef = types.clone();
        let fundef = quote! {&(Self::#name as fn(&_, #(#typdef),*) #output) };

        let fun = if types.is_empty() {
            quote! {
                ::log::debug!(stringify!(receiving for typed method #name : no params));
                let fun = #fundef;
                ::log::info!(stringify!(handling typed method #name));
                fun(base)
            }
        } else if types.len() == 1 {
            let typdsc = types.clone();
            quote! {
                ::log::debug!(stringify!(receiving for typed method #name : parsing params to #(#typdsc),*));
                let arg: #(#types),* = ::rpc_macro_support::parse_params(params)?;
                let fun = #fundef;
                ::log::info!(stringify!(handling typed method #name));
                fun(base, arg)
            }
        } else {
            let typdsc = types.clone();
            quote! {
                ::log::debug!(stringify!(receiving for typed method #name : parsing params to (#(#typdsc),*)));
                let args: (#(#types),*) = ::rpc_macro_support::parse_params(params)?;
                let fun = #fundef;
                ::log::info!(stringify!(handling typed method #name));
                fun(base, #(#args),*)
            }
        };

        Some(quote_spanned! {method.span()=>
            del.add_method(stringify!(#name), move |base, params| {
                #fun.map(|res| ::serde_json::to_value(&res).unwrap())
            });
        })
    });

    // create a new method for delegates
    let delegate: ImplItem = parse_quote! {
        /// Transform this into an `IoDelegate`, automatically wrapping the parameters.
        fn to_delegate<M: ::jsonrpc_core::Metadata>(self) -> ::jsonrpc_macros::IoDelegate<Self, M> {
            let mut del = ::jsonrpc_macros::IoDelegate::new(self.into());
            #(#delegates)*
            del
        }
    };

    // insert method
    in_impl.items.push(delegate);

    // rebuild token stream
    in_impl.into_token_stream().into()
}
