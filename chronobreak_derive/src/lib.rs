extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::token::Brace;
use syn::{Item, ItemMod, ItemUse, UsePath, UseTree};

#[proc_macro_attribute]
pub fn chronobreak(_attr: TokenStream, tokens: TokenStream) -> TokenStream {
    let item: Item = syn::parse(tokens).unwrap();
    let result = match &item {
        Item::Use(item) => {
            let mocked_item_uses = derive_item_use(item, false);
            quote! { #(#mocked_item_uses)* }
        }
        Item::Mod(item) => {
            let mocked_mod = derive_item_mod(item);
            quote! { #mocked_mod }
        }
        _ => panic! {"#[chronobreak] may only be applied to uses and modules"},
    };
    result.into()
}

fn derive_item_mod(item_mod: &ItemMod) -> ItemMod {
    let mut result = item_mod.clone();
    result.content = derive_mod_content(&item_mod.content);
    result
}

type ModContent = Option<(Brace, Vec<Item>)>;

fn derive_mod_content(content: &ModContent) -> ModContent {
    content
        .as_ref()
        .map(|(brace, items)| (*brace, items.iter().flat_map(&derive_mod_item).collect()))
}

fn derive_mod_item(item: &Item) -> impl std::iter::IntoIterator<Item = Item> {
    match item {
        Item::Use(item) => derive_item_use(item, true),
        item => vec![item.clone()],
    }
}

fn derive_item_use(item_use: &ItemUse, force_pub: bool) -> Vec<Item> {
    let attrs = &item_use.attrs;
    let attrs = quote! {#(#attrs)*};
    let vis = if force_pub {
        syn::parse(quote! {pub }.into()).unwrap()
    } else {
        item_use.vis.clone()
    };
    let use_path = match &item_use.tree {
        UseTree::Path(use_path) => use_path,
        _ => unimplemented! {},
    };
    let mocked_use_path = into_mocked_use_path(&use_path);
    vec![
        syn::parse(
            quote! {
                #[cfg(test)]
                #attrs #vis use ::#mocked_use_path;
            }
            .into(),
        )
        .unwrap(),
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
