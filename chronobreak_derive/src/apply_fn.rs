use proc_macro::TokenStream;
use proc_macro_error::*;
use quote::quote;
use syn::{parse::Parser, parse_quote, punctuated::Punctuated, Expr, Pat, Token};

pub fn derive(input: TokenStream) -> TokenStream {
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
    let (actual_pats, actual_body) = parse_closure_expr(exprs.next().unwrap());
    let (mocked_pats, mocked_body) = if let Some(expr) = exprs.next() {
        parse_closure_expr(expr)
    } else {
        (actual_pats.clone(), actual_body.clone())
    };
    (quote! {
        if crate::clock::is_mocked() {
            if let (#(crate::mock::Mock::Mocked(#mocked_pats),)*) = #args {
                #mocked_body
            } else {
                panic! {"expected a mocked value"}
            }
        } else {
            if let (#(crate::mock::Mock::Actual(#actual_pats),)*) = #args {
                #actual_body
            } else {
                panic! {"expected a non-mocked value"}
            }
        }
    })
    .into()
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
