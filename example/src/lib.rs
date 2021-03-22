#![allow(dead_code)]

#[allow(unused_imports)]
#[macro_use(chronobreak)]
extern crate chronobreak;

#[chronobreak]
use std::thread;
#[chronobreak]
use std::time::*;

fn return_at<T>(time: Instant, t: T) -> T {
    if Instant::now() < time {
        thread::sleep(time.saturating_duration_since(Instant::now()));
    }
    t
}

#[cfg(test)]
mod tests {
    use super::*;

    #[chronobreak::test]
    fn test_return_at() {
        let return_time = Instant::now() + Duration::from_secs(1);
        return_at(return_time, 0);
        assert_eq! {Instant::now(), return_time};
    }
}
