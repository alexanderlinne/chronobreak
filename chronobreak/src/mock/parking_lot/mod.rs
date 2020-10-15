use crate::clock;
use crate::mock::std::time::*;
use std::ops::{Deref, DerefMut};

pub use parking_lot::*;

type MutexData<T> = (T, Duration);

#[derive(Debug)]
pub struct Mutex<T> {
    mutex: parking_lot::Mutex<MutexData<T>>,
}

impl<T> Mutex<T> {
    pub fn new(val: T) -> Self {
        Self {
            mutex: parking_lot::Mutex::new((val, Duration::default())),
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

impl<'a, T> Deref for MutexGuard<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.guard.deref().0
    }
}

impl<'a, T> DerefMut for MutexGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.guard.deref_mut().0
    }
}

impl<'a, T> Drop for MutexGuard<'a, T> {
    fn drop(&mut self) {
        if clock::is_mocked() {
            self.guard.1 = Instant::now().into();
        }
    }
}

#[derive(Debug)]
pub struct Condvar {
    condvar: parking_lot::Condvar,
    time_sync: Mutex<()>,
}

impl Default for Condvar {
    fn default() -> Self {
        Self {
            condvar: parking_lot::Condvar::default(),
            time_sync: Mutex::new(()),
        }
    }
}

impl Condvar {
    pub fn new() -> Self {
        Self {
            condvar: parking_lot::Condvar::new(),
            time_sync: Mutex::new(()),
        }
    }

    pub fn notify_one(&self) -> bool {
        if clock::is_mocked() {
            self.time_sync.lock();
        }
        let result = self.condvar.notify_one();
        if clock::is_mocked() {
            self.time_sync.lock();
        }
        result
    }

    pub fn notify_all(&self) -> usize {
        if clock::is_mocked() {
            self.time_sync.lock();
        }
        let result = self.condvar.notify_all();
        if clock::is_mocked() {
            self.time_sync.lock();
        }
        result
    }

    pub fn wait<T>(&self, mutex_guard: &mut MutexGuard<T>) {
        if clock::is_mocked() {
            self.time_sync.lock();
        }
        self.condvar.wait(&mut mutex_guard.guard);
        if clock::is_mocked() {
            self.time_sync.lock();
        }
    }
}
