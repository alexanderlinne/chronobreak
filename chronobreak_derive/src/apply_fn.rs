use proc_macro::TokenStream;
use proc_macro_error::*;
use quote::quote;
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
        parse_quote! {crate::mock::Mock::Actual},
        &args,
        actual,
        "expected a non-mocked value",
        map,
    );
    let mocked_if_let = create_if_let(
        parse_quote! {crate::mock::Mock::Mocked},
        &args,
        exprs.next().unwrap_or(actual),
        "expected a mocked value",
        map,
    );
    (quote! {
        if crate::clock::is_mocked() {
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
) -> Expr {
    let (pats, body) = parse_closure_expr(closure);
    let if_let = parse_quote! {
        if let (#(#match_path(#pats),)*) = #args {
            #body
        } else {
            panic! {#error_msg}
        }
    };
    if map {
        parse_quote! {
            #match_path(#if_let)
        }
    } else {
        if_let
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
