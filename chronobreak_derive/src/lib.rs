extern crate proc_macro;

use darling::FromMeta;
use proc_macro::TokenStream;
use proc_macro_error::*;
use syn::{parse_macro_input, AttributeArgs};

fn parse_args<ArgStruct>(args: AttributeArgs) -> Result<ArgStruct, TokenStream>
where
    ArgStruct: FromMeta,
{
    ArgStruct::from_list(&args).map_err(|err| err.write_errors().into())
}

mod apply_fn;
mod chronobreak_attr;
mod constants_fn;
mod test_attr;

/// Enables the mock on an import or a group of imports.
///
/// This is a convenience macro for mocking imports. It causes the use
/// statement to be replaced by the mocked version when in test
/// configuration.
///
/// It can also be applied to inline modules, which will mock all imports in the
/// top-level module. This allows to group all imports that should be mocked.
///
/// # Examples
///
/// ```no_run
/// # use chronobreak::chronobreak;
/// #[chronobreak]
/// use std::thread::spawn;
///
/// #[chronobreak]
/// mod mock {
///     pub use std::sync::atomic::{AtomicUsize, Ordering};
///     pub use std::sync::Arc;
///     pub use std::time;
/// }
/// use mock::*;
/// ```
#[proc_macro_attribute]
#[proc_macro_error]
pub fn chronobreak(args: TokenStream, tokens: TokenStream) -> TokenStream {
    let args = parse_macro_input! {args as AttributeArgs};
    match chronobreak_attr::derive(args, tokens) {
        Ok(stream) => stream,
        Err(err) => err,
    }
}

/// Enables an (async) test function with a mocked clock.
///
/// Async tests require [async-std](https://crates.io/crates/async-std) as a
/// dependency.
///
/// # Examples
///
/// ```no_run
/// #[chronobreak::test]
/// fn test() {
///     // [...]
///     clock::advance(Duration::from_millis(1));
///     // [...]
/// }
///
/// #[chronobreak::test]
/// async fn async_test() {
///     // [...]
/// }
///
/// #[chronobreak::test(frozen)]
/// fn test_with_frozen_clock() {
///     // [...]
/// }
/// ```
#[proc_macro_attribute]
#[proc_macro_error]
pub fn test(args: TokenStream, tokens: TokenStream) -> TokenStream {
    let args = parse_macro_input! {args as AttributeArgs};
    match test_attr::derive(args, tokens) {
        Ok(stream) => stream,
        Err(err) => err,
    }
}

#[proc_macro]
#[proc_macro_error]
pub fn apply(input: TokenStream) -> TokenStream {
    apply_fn::derive(input, false)
}

#[proc_macro]
#[proc_macro_error]
pub fn map(input: TokenStream) -> TokenStream {
    apply_fn::derive(input, true)
}

#[proc_macro]
#[proc_macro_error]
pub fn constants(input: TokenStream) -> TokenStream {
    constants_fn::derive(input)
}
