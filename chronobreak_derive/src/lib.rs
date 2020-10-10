extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::parse::Parse;
use syn::{ItemUse, UsePath, UseTree};

struct Items {
    items: Vec<ItemUse>,
}

impl Parse for Items {
    fn parse(content: &syn::parse::ParseBuffer<'_>) -> std::result::Result<Self, syn::Error> {
        let mut items = Vec::new();
        while !content.is_empty() {
            items.push(content.parse()?);
        }
        Ok(Self { items })
    }
}

#[proc_macro]
pub fn chronobreak(tokens: TokenStream) -> TokenStream {
    let input: Items = syn::parse(tokens).unwrap();
    let input = input.items;
    let attrs = input.iter().map(|e| {
        let attrs = &e.attrs;
        quote! {#(#attrs)*}
    });
    let vis = input.iter().map(|e| &e.vis);
    let use_paths = input.iter().map(|item_use| match &item_use.tree {
        UseTree::Path(use_path) => use_path,
        _ => unimplemented! {},
    });
    let mocked_use_paths = use_paths.clone().map(&into_mocked_use_path);
    let result = quote! {
        #(
            #[cfg(test)]
            #attrs #vis use ::#mocked_use_paths;
            #[cfg(not(test))]
            #attrs #vis use #use_paths;
        )*
    };
    result.into()
}

fn into_mocked_use_path(use_path: &UsePath) -> proc_macro2::TokenStream {
    quote! {kled_mock::mock:: #use_path}
}
