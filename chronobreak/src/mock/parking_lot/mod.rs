use crate::clock;
use parking_lot;
use crate::mock::std::time::*;

pub use parking_lot::*;

type MutexData<T> = (T, Duration);

#[derive(Debug)]
pub struct Mutex<T> {
    mutex: parking_lot::Mutex<MutexData<T>>,
}

impl<T> Mutex<T> {
    pub fn new(val: T) -> Self {
        Self {
            mutex: parking_lot::Mutex::new((val, Duration::default()))
        }
    }

    pub fn lock(&self) -> MutexGuard<'_, T> {
        self.mutex.lock().into()
    }

    pub fn try_lock(&self) -> Option<MutexGuard<'_, T>> {
        self.mutex.try_lock().map(&Into::into)
    }
}

pub struct MutexGuard<'a, T> {
    guard: parking_lot::MutexGuard<'a, MutexData<T>>,
}

impl<'a, T> From<parking_lot::MutexGuard<'a, MutexData<T>>> for MutexGuard<'a, T> {
    fn from(guard: parking_lot::MutexGuard<'a, MutexData<T>>) -> Self {
        if clock::is_mocked() {
            clock::unfreeze_advance_to(guard.1.into());
        }
        MutexGuard { guard }
    }
}

impl<'a, T> Drop for MutexGuard<'a, T> {
    fn drop(self) {
        if clock::is_mocked() {
            self.guard.1 = Instant::now().into();
        }
    }
}

#[derive(Debug)]
pub struct Condvar {
    condvar: parking_lot::Condvar,
    time: Mutex<Instant>,
}

impl Default for Condvar {
    fn default() -> Self {
        Self {
            condvar: parking_lot::Condvar::default(),
            time: Mutex::new(Instant::now()),
        }
    }
}

impl Condvar {
    pub fn new() -> Self {
        Self { condvar: parking_lot::Condvar::new(), time: Mutex::new(Instant::now()) }
    }

    pub fn notify_one(&self) -> bool {
        if clock::is_mocked() {
            unimplemented! {}
        } else {
            self.condvar.notify_one()
        }
    }

    pub fn notify_all(&self) -> usize {
        if clock::is_mocked() {
            unimplemented! {}
        } else {
            self.condvar.notify_all()
        }
    }

    pub fn wait<T: ?Sized>(&self, mutex_guard: &mut MutexGuard<T>) {
        if clock::is_mocked() {
            unimplemented! {}
        } else {
            self.condvar.wait(mutex_guard.into())
        }
    }
}
