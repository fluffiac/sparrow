use std::time::Duration;

pub mod builder;

pub struct Interval {
    timer: async_io::Timer,
}

impl Interval {
    pub fn every(dur: Duration) -> Self {
        Self {
            timer: async_io::Timer::interval(dur),
        }
    }

    pub async fn wait(&mut self) {
        use futures_lite::stream::StreamExt;

        self.timer.next().await;
    }
}

// using ! directly is unstable, but there is a workaround
mod trick {
    pub trait HasOutput {
        type Output;
    }

    impl<O> HasOutput for fn() -> O {
        type Output = O;
    }

    type F = fn() -> !;

    pub type Never = <F as HasOutput>::Output;
}

pub use trick::Never;
