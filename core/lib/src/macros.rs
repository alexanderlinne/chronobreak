/// Asserts that the current local time is equal to an expression.
///
/// On panic, this macro will print the expected and actual local time.
#[macro_export]
macro_rules! assert_clock_eq {
    ($instant:expr) => ({
        match (&($instant),) {
            (instant,) => {
                if !(*instant == ::chronobreak::mock::std::time::Instant::now()) {
                    panic!(r#"clock assertion failed: `(expected == actual)`
 expected: `{:?}`,
   actual: `{:?}`"#, &*instant, ::chronobreak::mock::std::time::Instant::now())
                }
            }
        }
    });
    ($instant:expr,) => ({
        $crate::assert_clock_eq!($instant)
    });
    ($instant:expr, $($arg:tt)+) => ({
        match (&($instant),) {
            (instant,) => {
                if !(*instant == ::chronobreak::mock::std::time::Instant::now()) {
                    panic!(r#"clock assertion failed: `(expected == actual)`
 expected: `{:?}`,
   actual: `{:?}`: {}"#, &*instant, ::chronobreak::mock::std::time::Instant::now(),
                           $crate::format_args!($($arg)+))
                }
            }
        }
    });
}
