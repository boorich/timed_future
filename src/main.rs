use pin_project::pin_project;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::{Duration, Instant};
use tokio::time; // 0.2

/// A wrapper around a Future which adds timing data.
#[pin_project]
struct Timed<Fut, F>
where
    Fut: Future,
    F: FnMut(&Fut::Output, Duration),
{
    #[pin]
    inner: Fut,
    f: F,
    start: Option<Instant>,
}

impl<Fut, F> Future for Timed<Fut, F>
where
    Fut: Future,
    F: FnMut(&Fut::Output, Duration),
{
    type Output = Fut::Output;

    fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        let this = self.project();
        let start = this.start.get_or_insert_with(Instant::now);

        match this.inner.poll(cx) {
            // If the inner future is still pending, this wrapper is still pending.
            Poll::Pending => Poll::Pending,

            // If the inner future is done, measure the elapsed time and finish this wrapper future.
            Poll::Ready(v) => {
                let elapsed = start.elapsed();
                (this.f)(&v, elapsed);

                Poll::Ready(v)
            }
        }
    }
}

trait TimedExt: Sized + Future {
    fn timed<F>(self, f: F) -> Timed<Self, F>
    where
        F: FnMut(&Self::Output, Duration),
    {
        Timed {
            inner: self,
            f,
            start: None,
        }
    }
} 

// All futures can use the `.timed` method defined above
impl<F: Future> TimedExt for F {}

#[tokio::main]
async fn main() {
    let when = Instant::now() + Duration::from_millis(100);

    let fut = time::sleep_until(when.into())
        .timed(|res, elapsed| println!("{:?} elapsed, got {:?}", elapsed, res));
    fut.await;
}