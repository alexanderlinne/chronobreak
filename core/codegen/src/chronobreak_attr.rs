use darling::FromMeta;
use proc_macro::TokenStream;
use proc_macro_error::*;
use quote::{format_ident, quote};
use std::convert::TryInto;
use syn::{parse_quote, token::Brace, AttributeArgs, Item, ItemMod, ItemUse, UsePath, UseTree};

#[derive(FromMeta)]
struct Args {}

impl TryInto<Args> for AttributeArgs {
    type Error = TokenStream;

    fn try_into(self) -> Result<Args, Self::Error> {
        super::parse_args(self)
    }
}

pub fn derive(args: AttributeArgs, tokens: TokenStream) -> Result<TokenStream, TokenStream> {
    let items = match syn::parse(tokens).unwrap() {
        Item::Use(item) => derive_item_use(&args.try_into()?, &item),
        Item::Mod(item) => vec![derive_item_mod(&args.try_into()?, &item)],
        item => abort! {item, "#[chronobreak] may only be applied to use statements and modules"},
    };
    Ok((quote! {#(#items)*}).into())
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
    let mocked_use_path = into_mocked_use_path(use_path);
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
    let ident = format_ident! {"chronobreak_{}", use_path.ident};
    let colon2_token = use_path.colon2_token;
    let tree = &use_path.tree;
    quote! {#ident #colon2_token #tree}
}
