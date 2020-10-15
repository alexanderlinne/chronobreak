use crate::clock;
use futures::{
    executor,
    future::{Future, FutureObj},
    io::Error,
    task::{Spawn, SpawnError},
};

pub use futures::executor::*;

#[derive(Default, Debug)]
pub struct ThreadPoolBuilder {
    builder: executor::ThreadPoolBuilder,
    after_start_called: bool,
}

impl ThreadPoolBuilder {
    pub fn new() -> ThreadPoolBuilder {
        Self {
            builder: executor::ThreadPool::builder(),
            after_start_called: false,
        }
    }

    pub fn pool_size(&mut self, size: usize) -> &mut ThreadPoolBuilder {
        self.builder.pool_size(size);
        self
    }

    pub fn stack_size(&mut self, stack_size: usize) -> &mut ThreadPoolBuilder {
        self.builder.stack_size(stack_size);
        self
    }

    pub fn name_prefix<S>(&mut self, name_prefix: S) -> &mut ThreadPoolBuilder
    where
        S: Into<String>,
    {
        self.builder.name_prefix(name_prefix);
        self
    }

    pub fn after_start<F>(&mut self, f: F) -> &mut ThreadPoolBuilder
    where
        F: Fn(usize) + Send + Sync + 'static,
    {
        self.after_start_called = true;
        let handle = clock::handle();
        self.builder.after_start(move |id| {
            clock::register_thread(handle.clone());
            f(id)
        });
        self
    }

    pub fn before_stop<F>(&mut self, f: F) -> &mut ThreadPoolBuilder
    where
        F: Fn(usize) + Send + Sync + 'static,
    {
        self.builder.before_stop(f);
        self
    }

    pub fn create(&mut self) -> Result<ThreadPool, Error> {
        if !self.after_start_called {
            let handle = clock::handle();
            self.builder.after_start(move |_| {
                clock::register_thread(handle.clone());
            });
        }
        self.builder.create().map(|pool| ThreadPool { pool })
    }
}

#[derive(Clone, Debug)]
pub struct ThreadPool {
    pool: executor::ThreadPool,
}

impl Spawn for ThreadPool {
    fn spawn_obj(&self, future: FutureObj<'static, ()>) -> Result<(), SpawnError> {
        self.pool.spawn_obj(future)
    }
}

impl ThreadPool {
    pub fn new() -> Result<ThreadPool, Error> {
        ThreadPoolBuilder::new().create()
    }

    pub fn builder() -> ThreadPoolBuilder {
        ThreadPoolBuilder::new()
    }

    pub fn spawn_obj_ok(&self, future: FutureObj<'static, ()>) {
        self.pool.spawn_obj_ok(future)
    }

    pub fn spawn_ok<Fut>(&self, future: Fut)
    where
        Fut: Future<Output = ()> + Send + 'static,
    {
        self.pool.spawn_ok(future)
    }
}
