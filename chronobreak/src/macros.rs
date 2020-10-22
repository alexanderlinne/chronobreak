/// Asserts that the current local time is equal to an expression.
///
/// On panic, this macro will print the expected and actual local time.
#[macro_export]
macro_rules! assert_clock_eq {
    ($dur:expr) => ({
        match (&($dur),) {
            (dur,) => {
                if !(*dur == ::chronobreak::clock::get()) {
                    panic!(r#"clock assertion failed: `(expected == actual)`
 expected: `{:?}`,
   actual: `{:?}`"#, &*dur, ::chronobreak::clock::get())
                }
            }
        }
    });
    ($dur:expr,) => ({
        $crate::assert_clock_eq!($dur)
    });
    ($dur:expr, $($arg:tt)+) => ({
        match (&($dur),) {
            (dur,) => {
                if !(*dur == ::chronobreak::clock::get()) {
                    panic!(r#"clock assertion failed: `(expected == actual)`
 expected: `{:?}`,
   actual: `{:?}`: {}"#, &*dur, ::chronobreak::clock::get(),
                           $crate::format_args!($($arg)+))
                }
            }
        }
    });
}
