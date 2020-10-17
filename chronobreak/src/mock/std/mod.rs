pub use std::{
    alloc, any, arch, array, ascii, assert, assert_eq, assert_ne, borrow, boxed, cell, cfg, char,
    clone, cmp, collections, column, compile_error, concat, convert, dbg, debug_assert,
    debug_assert_eq, debug_assert_ne, default, env, eprint, eprintln, error, f32, f64, ffi, file,
    fmt, format, format_args, fs, future, hash, hint, i128, i16, i32, i64, i8, include,
    include_bytes, include_str, io, is_x86_feature_detected, isize, iter, line, marker, matches,
    mem, module_path, net, num, ops, option, option_env, os, panic, path, pin, prelude, primitive,
    print, println, process, ptr, rc, result, slice, str, string, stringify, task, thread_local,
    todo, u128, u16, u32, u64, u8, unimplemented, unreachable, usize, vec, write, writeln,
};

/// **Mock** of [`std::sync`](https://doc.rust-lang.org/std/sync/index.html)
pub mod sync;

/// **Mock** of [`std::thread`](https://doc.rust-lang.org/std/thread/index.html)
pub mod thread;

/// **Mock** of [`std::time`](https://doc.rust-lang.org/std/time/index.html)
pub mod time;
