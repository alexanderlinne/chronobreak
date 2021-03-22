use crate::clock;
use std::fmt;
use std::ops::{Deref, DerefMut};

pub use parking_lot::{
    const_fair_mutex, const_mutex, const_reentrant_mutex, const_rwlock, lock_api, FairMutex,
    FairMutexGuard, MappedFairMutexGuard, MappedMutexGuard, MappedReentrantMutexGuard,
    MappedRwLockReadGuard, MappedRwLockWriteGuard, Once, OnceState, RawFairMutex, RawMutex,
    RawRwLock, RawThreadId, ReentrantMutex, ReentrantMutexGuard, RwLock, RwLockReadGuard,
    RwLockUpgradableReadGuard, RwLockWriteGuard, WaitTimeoutResult,
};

struct MutexData<T>(T, clock::SyncHandle);

impl<T: fmt::Debug> fmt::Debug for MutexData<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        self.0.fmt(f)
    }
}

/// **Mock** of [`parking_lot::Mutex`](https://docs.rs/parking_lot/0.11.0/parking_lot/type.Mutex.html)
#[derive(Debug)]
pub struct Mutex<T> {
    mutex: parking_lot::Mutex<MutexData<T>>,
}

impl<T> Mutex<T> {
    pub fn new(val: T) -> Self {
        Self {
            mutex: parking_lot::Mutex::new(MutexData(val, clock::sync_handle())),
        }
    }

    pub fn lock(&self) -> MutexGuard<'_, T> {
        self.mutex.lock().into()
    }

    pub fn try_lock(&self) -> Option<MutexGuard<'_, T>> {
        self.mutex.try_lock().map(&Into::into)
    }
}

/// **Mock** of [`parking_lot::MutexGuard`](https://docs.rs/parking_lot/0.11.0/parking_lot/type.MutexGuard.html)
pub struct MutexGuard<'a, T> {
    guard: parking_lot::MutexGuard<'a, MutexData<T>>,
}

impl<'a, T> From<parking_lot::MutexGuard<'a, MutexData<T>>> for MutexGuard<'a, T> {
    fn from(guard: parking_lot::MutexGuard<'a, MutexData<T>>) -> Self {
        if clock::is_mocked() {
            let _guard = clock::unfreeze_scoped();
            clock::sync_with(guard.1);
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
            self.guard.1 = clock::sync_handle();
        }
    }
}

/// **Mock** of [`parking_lot::Condvar`](https://docs.rs/parking_lot/0.11.0/parking_lot/struct.Condvar.html)
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

impl fmt::Debug for Condvar {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        self.condvar.fmt(f)
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
