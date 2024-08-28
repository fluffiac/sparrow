use std::future::Future;
use std::io;

use async_executor::Executor as AsyncExecutor;
use async_io::block_on;
use kanal::{bounded, Receiver, Sender};

use crate::util::Never;
use crate::MemReader;

type BoxFn<T> = Box<(dyn Fn(&T) + Send + Sync)>;

pub struct Task<T: PartialEq + Clone> {
    value: Option<T>,
    callbacks: Vec<BoxFn<T>>,
}

impl<T: PartialEq + Clone> Task<T> {
    pub fn update(&mut self, value: &T) {
        if self.value.as_ref() != Some(value) {
            self.value = Some(value.clone());
            self.callbacks.iter().for_each(|f| f(value));
        }
    }
}

/// A trait that is applied to all reader functions. This is used when
/// generating the struct and impls for the user's memory reader struct
pub trait TaskFn<T> {}

impl<T: PartialEq + Clone + Send + Sync, F, Fut> TaskFn<T> for F
where
    F: Fn(MemReader, Task<T>) -> Fut,
    Fut: Future<Output = io::Result<Never>> + Send + 'static,
{
}

#[derive(Default)]
pub struct TaskBuilder<T> {
    callbacks: Vec<BoxFn<T>>,
}

impl<T: PartialEq + Clone + Send + Sync> TaskBuilder<T> {
    pub fn on_update(&mut self, func: BoxFn<T>) {
        self.callbacks.push(func);
    }

    pub fn into_task(self) -> Task<T> {
        Task {
            value: None,
            callbacks: self.callbacks,
        }
    }
}

pub struct Executor {
    ex: AsyncExecutor<'static>,
    err_tx: Sender<io::Error>,
    err_rx: Receiver<io::Error>,
}

impl Executor {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        let (err_tx, err_rx) = bounded(0);

        Self {
            ex: AsyncExecutor::new(),
            err_tx,
            err_rx,
        }
    }

    pub fn spawn(&self, fut: impl Future<Output = io::Result<Never>> + Send + 'static) {
        let err_tx = self.err_tx.clone();

        let fut = async move {
            let _ = err_tx.send(fut.await.unwrap_err());
        };

        self.ex.spawn(fut).detach();
    }

    pub fn into_runner(self) -> Runner {
        Runner {
            ex: self.ex,
            error: self.err_rx,
        }
    }
}

pub struct Runner {
    ex: AsyncExecutor<'static>,
    error: Receiver<io::Error>,
}

impl Runner {
    pub fn run_on_threads(self, threads: usize) -> io::Error {
        if threads == 0 {
            return io::Error::new(
                io::ErrorKind::InvalidInput,
                "threads must be greater than 0",
            );
        }

        let (tx, rx) = bounded::<()>(0);

        std::thread::scope(|s| {
            for _ in 0..threads {
                s.spawn(|| block_on(self.ex.run(rx.as_async().recv())));
            }

            let err = self.error.recv().unwrap();
            drop(tx);
            err
        })
    }
}

/// Generates a struct and impls for each memory reader passed into a faux
/// "struct builder".
macro_rules! mem_builder {
    // matches a MemBuilder builder object and extracts the types and names of
    // the reader functions. also constructs a struct and checks the builder
    // against the struct which is used to forward error messages to the call
    // site.
    (@check $mem:ident MemBuilder::new()$(.with::<$sends:ty>($name:ident))*.build() $($tt:tt)*) => {
        $crate::builder::mem_builder!(@generate $mem $($sends, $name,)*);

        #[allow(unused)]
        fn __check() {
            struct MemBuilder;
            impl MemBuilder {
                fn new() -> Self { Self }
                fn with<T>(self, item: impl $crate::builder::TaskFn<T>) -> Self { Self }
                fn build(self) {}
            }

            $($tt)*
        }
    };
    // error branch. this doesn't have all the necessary type info, so it will
    // forward an error message to the caller.
    (@check $mem:ident MemBuilder::new()$(.with$(::<$sends:ty>)?($name:ident))*.build() $($tt:tt)*) => {
        mod __generated {
            #[allow(unused)]
            fn __check() {
                struct MemBuilder;
                impl MemBuilder {
                    fn new() -> Self { Self }
                    fn with<T>(self, item: impl std::any::Any) -> Self { Self }
                    fn build(self) {}
                }

                $($tt)*
            }
        }
    };
    // Make a struct
    (@generate $mem:ident $($sends:ty, $name:ident,)*) => {
        pub struct $mem {
            $(pub $name: $crate::builder::TaskBuilder<$sends>),*
        }

        impl $mem {
            #[allow(clippy::new_without_default)]
            pub fn new() -> Self {
                Self {
                    $(
                        $name: $crate::builder::TaskBuilder::default(),
                    )*
                }
            }

            $(
                concat_idents::concat_idents!(name = on_, $name, _update {
                    pub fn name<F: Fn(&$sends) + Send + Sync + 'static>(mut self, func: F) -> Self {
                        self.$name.on_update(Box::new(func));
                        self
                    }
                });
            )*

            pub fn read_with(self, mr: $crate::MemReader) -> $crate::builder::Runner {
                let exec = $crate::builder::Executor::new();

                $(
                    exec.spawn($name(mr.clone(), self.$name.into_task()));
                )*

                exec.into_runner()
            }
        }
    };
    // main macro invocation
    (pub struct $mem:ident = $($tt:tt)+) => {
        $crate::builder::mem_builder!(@check $mem $($tt)+ $($tt)+);
    }
}

pub(crate) use mem_builder;
