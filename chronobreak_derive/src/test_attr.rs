use darling::FromMeta;
use proc_macro::TokenStream;
use proc_macro_error::*;
use quote::quote;
use std::convert::TryInto;
use syn::{parse_quote, AttributeArgs, Item, ItemFn};

#[derive(FromMeta)]
struct FnArgs {
    #[darling(default)]
    frozen: bool,
}

impl TryInto<FnArgs> for AttributeArgs {
    type Error = TokenStream;

    fn try_into(self) -> Result<FnArgs, Self::Error> {
        super::parse_args(self)
    }
}

pub fn derive(args: AttributeArgs, tokens: TokenStream) -> Result<TokenStream, TokenStream> {
    let item: Item = syn::parse(tokens).unwrap();
    let items = match &item {
        Item::Fn(item) => vec![derive_item_fn(&args.try_into()?, item)],
        item => abort! {item, "#[test] may only be applied to functions"},
    };
    Ok((quote! {#(#items)*}).into())
}

fn derive_item_fn(args: &FnArgs, item_fn: &ItemFn) -> Item {
    let test_attr = if item_fn.sig.asyncness.is_some() {
        quote! {#[async_std::test]}
    } else {
        quote! {#[test]}
    };
    let attrs = &item_fn.attrs;
    let vis = &item_fn.vis;
    let sig = &item_fn.sig;
    let mock_fn = if args.frozen {
        quote! {frozen}
    } else {
        quote! {mock}
    };
    let stmts = &item_fn.block.stmts;
    Item::Fn(parse_quote! {
        #test_attr
        #(#attrs)*
        #vis #sig {
            use ::chronobreak::clock;
            let _clock = clock::#mock_fn().unwrap();
            #(#stmts)*
        }
    })
}
