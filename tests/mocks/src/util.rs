macro_rules! impl_clone {
    ($name:ident, $obj:expr) => {
        paste! {
            #[test]
            #[allow(unused_variables, unused_must_use)]
            fn [<$name _impls_clone>]() {
                let obj = $obj;
                let obj1 = obj.clone();
                // make clippy happy
                drop(obj);
                drop(obj1);
            }
        }
    };
}

macro_rules! impl_debug {
    ($name:ident, $obj:expr) => {
        paste! {
            #[test]
            fn [<$name _impls_debug>]() {
                let obj = $obj;
                format!("{:?}", obj);
            }
        }
    };
}

macro_rules! impl_default {
    ($name:ident, $ty:ty) => {
        paste! {
            #[test]
            fn [<$name _impls_default>]() {
                let _ = $ty::default();
            }
        }
    };
}
