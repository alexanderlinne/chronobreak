use proc_macro::TokenStream;
use proc_macro_error::*;
use quote::quote;
use syn::{parse::Parser, punctuated::Punctuated, Expr, Token};

pub fn derive(input: TokenStream) -> TokenStream {
    let exprs = <Punctuated<Expr, Token![,]>>::parse_terminated
        .parse(input)
        .unwrap();
    let tuples: Vec<_> = exprs
        .iter()
        .map(|expr| {
            if let Expr::Tuple(tuple) = expr {
                if tuple.elems.len() != 3 {
                    abort! {tuple.elems, "constants! expected a 3-tuple here:"};
                }
                tuple
            } else {
                abort! {expr, "constants! expected a tuple here:"};
            }
        })
        .collect();
    let constants_count = tuples.len();
    let ids = (0usize..).take(tuples.len());
    let idents = tuples.iter().map(|tuple| tuple.elems.first());
    let actual_exprs = tuples.iter().map(|tuple| tuple.elems.iter().nth(1));
    let mocked_exprs = tuples.iter().map(|tuple| tuple.elems.iter().nth(2));
    (quote! {
        #(
            pub const #idents: Self = Self(crate::mock::Mock::constant(#ids));
        )*
        const __CHRONOBREAK_CONSTANTS: [(Self, Self); #constants_count] = [
            #(
                (
                    Self(crate::mock::Mock::actual(#actual_exprs)),
                    Self(crate::mock::Mock::mocked(#mocked_exprs))
                )
            )*
        ];

        fn __chronobreak_constants(inst: &Self) -> &Self {
            if let crate::mock::Mock::Constant(id) = inst.0 {
                if crate::clock::is_mocked() {
                    return &Self::__CHRONOBREAK_CONSTANTS[id].1
                } else {
                    return &Self::__CHRONOBREAK_CONSTANTS[id].0
                }
            };
            inst
        }

        fn __chronobreak_constants_mut(inst: &mut Self) -> &mut Self {
            *inst = *Self::__chronobreak_constants(inst);
            inst
        }
    })
    .into()
}
