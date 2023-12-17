use std::future::Future;

use kanal::AsyncSender;

use crate::hdmem::{Mem, MemError, Result};
use crate::pollers::Event;

pub trait Poll {
    fn poll(mem: &Mem, tx: AsyncSender<Event>) -> impl Future<Output = Result<!>> + Send;
}

pub use runner::PollRunner;

mod runner {
    use std::future::Future;
    use std::pin::Pin;

    use super::*;
    use crate::util::as_static;

    // inner poll wraps outer poll so it can be a trait object
    pub trait Poll: Send + Sync {
        fn poll_fut<'a>(
            &self,
            mem: &'a Mem,
            tx: AsyncSender<Event>,
        ) -> Pin<Box<dyn Future<Output = Result<!>> + Send + 'a>>;
    }

    impl<T: super::Poll + Send + Sync + 'static> Poll for T {
        fn poll_fut<'a>(
            &self,
            mem: &'a Mem,
            tx: AsyncSender<Event>,
        ) -> Pin<Box<dyn Future<Output = Result<!>> + Send + 'a>> {
            Box::pin(T::poll(mem, tx))
        }
    }

    #[derive(Default)]
    pub struct PollRunner(Vec<Box<dyn Poll>>);

    impl PollRunner {
        pub fn new() -> Self {
            Self::default()
        }

        pub fn add(&mut self, poller: impl Poll + 'static) {
            self.0.push(Box::new(poller));
        }

        pub async fn poll(&self, mem: &Mem, tx: &AsyncSender<Event>) -> MemError {
            use futures::future::select_all;

            // this is safe, so long as we...
            let mem = unsafe { as_static(mem) };

            let jh = self
                .0
                .iter()
                .map(|f| tokio::spawn(f.poll_fut(mem, tx.clone())));

            let (err, _, rest) = select_all(jh).await;

            for jh in rest {
                jh.abort();
                // ... await to avoid use after free
                drop(jh.await);
            }

            err.unwrap().unwrap_err()
        }
    }
}
