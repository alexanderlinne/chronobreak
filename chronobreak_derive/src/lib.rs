extern crate proc_macro;

use darling::FromMeta;
use proc_macro::TokenStream;
use proc_macro_error::*;
use quote::quote;
use std::convert::TryInto;
use syn::token::Brace;
use syn::{
    parse_macro_input, parse_quote, AttributeArgs, Item, ItemFn, ItemMod, ItemUse, UsePath, UseTree,
};

#[derive(FromMeta)]
struct Args {}

#[derive(FromMeta)]
struct FnArgs {
    #[darling(default)]
    frozen: bool,
}

impl TryInto<Args> for AttributeArgs {
    type Error = TokenStream;

    fn try_into(self) -> Result<Args, Self::Error> {
        parse_args(self)
    }
}

impl TryInto<FnArgs> for AttributeArgs {
    type Error = TokenStream;

    fn try_into(self) -> Result<FnArgs, Self::Error> {
        parse_args(self)
    }
}

#[proc_macro_attribute]
#[proc_macro_error]
pub fn chronobreak(args: TokenStream, tokens: TokenStream) -> TokenStream {
    let args = parse_macro_input! {args as AttributeArgs};
    match derive_chronobreak(args, tokens) {
        Ok(stream) => stream,
        Err(err) => err,
    }
}

fn derive_chronobreak(
    args: AttributeArgs,
    tokens: TokenStream,
) -> Result<TokenStream, TokenStream> {
    let items = match syn::parse(tokens).unwrap() {
        Item::Use(item) => derive_item_use(&args.try_into()?, &item),
        Item::Mod(item) => vec![derive_item_mod(&args.try_into()?, &item)],
        item => abort! {item, "#[chronobreak] may only be applied to use statements and modules"},
    };
    Ok((quote! {#(#items)*}).into())
}

fn parse_args<ArgStruct>(args: AttributeArgs) -> Result<ArgStruct, TokenStream>
where
    ArgStruct: FromMeta,
{
    ArgStruct::from_list(&args).map_err(|err| err.write_errors().into())
}

fn derive_item_mod(args: &Args, item_mod: &ItemMod) -> Item {
    let mut result = item_mod.clone();
    result.content = derive_mod_content(args, &item_mod.content);
    Item::Mod(result)
}

type ModContent = Option<(Brace, Vec<Item>)>;

fn derive_mod_content(args: &Args, content: &ModContent) -> ModContent {
    content.as_ref().map(|(brace, items)| {
        (
            *brace,
            items
                .iter()
                .flat_map(|e| derive_mod_item(args, e))
                .collect(),
        )
    })
}

fn derive_mod_item(args: &Args, item: &Item) -> impl std::iter::IntoIterator<Item = Item> {
    match item {
        Item::Use(item) => derive_item_use(args, item),
        item => vec![item.clone()],
    }
}

fn derive_item_use(_: &Args, item_use: &ItemUse) -> Vec<Item> {
    let attrs = &item_use.attrs;
    let attrs = quote! {#(#attrs)*};
    let vis = &item_use.vis;
    let use_path = match &item_use.tree {
        UseTree::Path(use_path) => use_path,
        _ => unimplemented! {},
    };
    let mocked_use_path = into_mocked_use_path(&use_path);
    vec![
        parse_quote! {
            #[cfg(test)]
            #attrs #vis use ::#mocked_use_path;
        },
        syn::parse(
            quote! {
                #[cfg(not(test))]
                #attrs #vis use #use_path;
            }
            .into(),
        )
        .unwrap(),
    ]
}

fn into_mocked_use_path(use_path: &UsePath) -> proc_macro2::TokenStream {
    quote! {chronobreak::mock:: #use_path}
}

#[proc_macro_attribute]
#[proc_macro_error]
pub fn test(args: TokenStream, tokens: TokenStream) -> TokenStream {
    let args = parse_macro_input! {args as AttributeArgs};
    match derive_test(args, tokens) {
        Ok(stream) => stream,
        Err(err) => err,
    }
}

fn derive_test(args: AttributeArgs, tokens: TokenStream) -> Result<TokenStream, TokenStream> {
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
    let freeze_stmt = if args.frozen {
        quote! {clock::freeze();}
    } else {
        quote! {}
    };
    let stmts = &item_fn.block.stmts;
    Item::Fn(parse_quote! {
        #test_attr
        #(#attrs)*
        #vis #sig {
            use ::chronobreak::clock;
            let _clock = clock::mocked().unwrap();
            #freeze_stmt
            #(#stmts)*
        }
    })
}
