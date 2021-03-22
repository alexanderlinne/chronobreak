use std::fmt::Debug;

pub fn is_clone<T: Clone>() {}
pub fn is_debug<T: Debug>() {}
pub fn is_default<T: Default>() {}
pub fn is_send<T: Send>() {}
pub fn is_sync<T: Sync>() {}
