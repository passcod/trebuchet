extern crate proc_macro as pm1;

use quote::{quote, quote_spanned, ToTokens};
use syn::spanned::Spanned;
use syn::{parse_macro_input, parse_quote, FnArg, ImplItem, ImplItemMethod, ItemImpl};

#[proc_macro]
pub fn rpc_impl_struct(input: pm1::TokenStream) -> pm1::TokenStream {
    let mut in_impl = parse_macro_input!(input as ItemImpl);

    // split impl into items we care about and the rest
    let (methods, others): (Vec<ImplItem>, Vec<ImplItem>) =
        in_impl.items.into_iter().partition(|item| match item {
            ImplItem::Method(_) => true,
            _ => false,
        });

    // extract methods
    let methods = methods.into_iter().map(|item| match item {
        ImplItem::Method(m) => m,
        _ => unreachable!(),
    });

    // transform method parameters
    let methods: Vec<ImplItemMethod> = methods
        .map(|method| {
            let inputs = &method.sig.decl.inputs;
            let args: Vec<&FnArg> = inputs
                .iter()
                .filter(|input| match input {
                    FnArg::Captured(_) => true,
                    _ => false,
                })
                .collect();

            if args.is_empty() {
                // no args (apart from self), no change needed
                return method;
            }

            let attrs = &method.attrs;
            let vis = &method.vis;
            let name = &method.sig.ident;
            let fn_token = &method.sig.decl.fn_token;
            let generics = &method.sig.decl.generics;
            let output = &method.sig.decl.output;
            let stmts = &method.block.stmts;

            let pars = inputs.clone().into_iter().flat_map(|input| {
                if let FnArg::Captured(arg) = input {
                    let name = &arg.pat;
                    Some(quote_spanned! {arg.span()=> #name })
                } else {
                    None
                }
            });

            let typs = inputs.clone().into_iter().flat_map(|input| {
                if let FnArg::Captured(arg) = input {
                    let ty = &arg.ty;
                    Some(quote_spanned! {arg.span()=> #ty })
                } else {
                    None
                }
            });

            let res: ImplItemMethod = parse_quote! {
                #(#attrs)*
                #vis #fn_token#generics #name(&self, params: ::jsonrpc_core::Params) #output {
                    let (#(#pars),*): (#(#typs),*) = ::rpc_macro_support::parse_params(params)?;
                    #(#stmts)*
                }
            };

            // let res: proc_macro2::TokenStream = res.into_token_stream();
            // panic!("{}", res);
            res
        })
        .collect();

    // generate delegated entries for methods we care about
    let delegates = methods.iter().map(|method| {
        let name = &method.sig.ident;
        let output = &method.sig.decl.output;
        let types = method.sig.decl.inputs.iter().filter_map(|input| {
            if let FnArg::Captured(arg) = input {
                Some(&arg.ty)
            } else {
                None
            }
        });

        Some(quote_spanned! {method.span()=>
            del.add_method(stringify!(#name), move |base, params| {
                ::jsonrpc_macros::WrapAsync::wrap_rpc(
                    &(Self::#name as fn(&_, #(#types),*) #output),
                    base,
                    params,
                )
            });
        })
    });

    // create a new method for delegates
    let delegate_tokens = quote! {
        /// Transform this into an `IoDelegate`, automatically wrapping the parameters.
        fn to_delegate<M: ::jsonrpc_core::Metadata>(self) -> ::jsonrpc_macros::IoDelegate<Self, M> {
            let mut del = ::jsonrpc_macros::IoDelegate::new(self.into());
            #(#delegates)*
            del
        }
    };

    // reparse it
    let delegate_pm1: pm1::TokenStream = delegate_tokens.into();
    let delegate = parse_macro_input!(delegate_pm1 as ImplItem);

    // remake items
    let methods: Vec<ImplItem> = methods.into_iter().map(ImplItem::Method).collect();

    // put everything together again
    let mut items = Vec::new();
    items.extend(methods);
    items.extend(others);
    items.push(delegate);

    // rebuild the impl
    in_impl.items = items;
    in_impl.into_token_stream().into()
}
