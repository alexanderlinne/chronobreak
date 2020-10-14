use chronobreak::clock;
use paste::paste;
#[chronobreak]
use std::time::*;

macro_rules! test_incompatible {
    ($op:ident, $var_qual:ident) => {
        paste! {
            #[test]
            #[should_panic]
            fn [<instant_ $op _incompatible_1>]() {
                let lhs = Instant::now();
                let _clock = clock::mocked().unwrap();
                let rhs = Instant::now();
                let _ =lhs.$op($var_qual!(rhs));
            }
            #[test]
            #[should_panic]
            fn [<instant_ $op _incompatible_2>]() {
                let lhs = Instant::now();
                let _clock = clock::mocked().unwrap();
                let rhs = Instant::now();
                let _ = rhs.$op($var_qual!(lhs));
            }
        }
    };
    ($op:ident) => {
        macro_rules! qualifier {
            ($v:ident) => {
                $v
            };
        }
        test_incompatible! {$op, qualifier}
    };
    (&$op:ident) => {
        macro_rules! qualifier {
            ($v:ident) => {
                &$v
            };
        }
        test_incompatible! {$op, qualifier}
    };
}

test_incompatible! {saturating_duration_since}
test_incompatible! {&cmp}
test_incompatible! {&partial_cmp}
test_incompatible! {&eq}
