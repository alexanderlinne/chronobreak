use proc_macro::TokenStream;
use proc_macro_error::*;
use quote::{format_ident, quote};
use syn::{parse::Parser, parse_quote, punctuated::Punctuated, Expr, ExprTuple, Pat, Path, Token};

pub fn derive(input: TokenStream, map: bool) -> TokenStream {
    let exprs = <Punctuated<Expr, Token![,]>>::parse_terminated
        .parse(input)
        .unwrap();
    if exprs.len() < 2 || exprs.len() > 3 {
        abort! {exprs, "apply! expectes 2 or 3 arguments"};
    }
    let mut exprs = exprs.iter();
    let args = match exprs.next().unwrap() {
        Expr::Tuple(tuple) => tuple.clone(),
        expr => parse_quote! {(#expr,)},
    };
    let actual = exprs.next().unwrap();
    let actual_if_let = create_if_let(
        parse_quote! {chronobreak::mock::Mock::Actual},
        &args,
        actual,
        "expected a non-mocked value",
        map,
    );
    let mocked_if_let = create_if_let(
        parse_quote! {chronobreak::mock::Mock::Mocked},
        &args,
        exprs.next().unwrap_or(actual),
        "expected a mocked value",
        map,
    );
    (quote! {
        if chronobreak::clock::is_mocked() {
            #mocked_if_let
        } else {
            #actual_if_let
        }
    })
    .into()
}

fn create_if_let(
    match_path: Path,
    args: &ExprTuple,
    closure: &Expr,
    error_msg: &str,
    map: bool,
) -> proc_macro2::TokenStream {
    let (pats, body) = parse_closure_expr(closure);
    if pats.len() != args.elems.len() {
        abort! {closure, "apply!: the count of povided arguments and arguments expected by the closure do not match"};
    }
    let mapped: Vec<_> = (0usize..)
        .zip(pats.iter())
        .zip(args.elems.iter())
        .map(|((id, pat), arg)| {
            let ident = format_ident!("__chronobreak_{}", id);
            match pat {
                Pat::Ident(ident_pat) => {
                    let arg_by_ref = if let Expr::Reference(_) = arg {
                        quote! {&}
                    } else {
                        quote! {}
                    };
                    let inv_arg_by_ref = if let Expr::Reference(_) = arg {
                        quote! {}
                    } else {
                        quote! {&}
                    };
                    let constants_expr = if ident_pat.mutability.is_some() {
                        quote! { let mut #ident = Self::__chronobreak_constants_mut(#arg); }
                    } else {
                        quote! { let #ident = Self::__chronobreak_constants(#inv_arg_by_ref #arg); }
                    };
                    (quote! {#arg_by_ref #ident}, constants_expr)
                }
                Pat::Wild(_) => (
                    quote! {#ident},
                    quote! { let #ident = Self::__chronobreak_constants(#arg); },
                ),
                _ => abort! {pat, "apply! currently only supports ident and wildcard patterns"},
            }
        })
        .collect();
    let idents = mapped.iter().map(|v| &v.0);
    let if_let = quote! {
        if let (#(#match_path(#pats),)*) = (#(#idents.0, )*) {
            #body
        } else {
            panic! {#error_msg}
        }
    };
    let constants_exprs = mapped.iter().map(|v| &v.1);
    if map {
        quote! {
            #(#constants_exprs)*
            #match_path(#if_let)
        }
    } else {
        quote! {
            #(#constants_exprs)*
            #if_let
        }
    }
}

fn parse_closure_expr(closure_expr: &syn::Expr) -> (Vec<&Pat>, Box<Expr>) {
    let closure = if let Expr::Closure(closure) = closure_expr {
        closure
    } else {
        abort! {closure_expr, "apply! expected a closure here:"};
    };
    if closure.inputs.len() != 1 {
        abort! {closure.inputs, "apply! expected a closure with exactly one argument here:"};
    }
    let pats = match closure.inputs.first().unwrap() {
        Pat::Tuple(tuple) => tuple.elems.iter().collect(),
        pat => vec![pat],
    };
    (pats, closure.body.clone())
}
