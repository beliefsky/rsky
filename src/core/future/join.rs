use core::{
    future::{Future, poll_fn},
    mem,
    pin::Pin,
    task::{Context, Poll},
};

pub fn join<Fut1, Fut2>(
    fut1: Fut1,
    fut2: Fut2,
) -> impl Future<Output = (Fut1::Output, Fut2::Output)>
where
    Fut1: Future,
    Fut2: Future,
{
    let futures = (MaybeDone::Future(fut1), MaybeDone::Future(fut2));
    async {
        let mut futures = futures;
        let mut futures = unsafe { Pin::new_unchecked(&mut futures) };
        poll_fn(move |cx| {
            let mut done = true;

            let fut = unsafe { futures.as_mut().map_unchecked_mut(|it| &mut it.0) };
            done &= fut.poll(cx).is_ready();

            let fut = unsafe { futures.as_mut().map_unchecked_mut(|it| &mut it.1) };
            done &= fut.poll(cx).is_ready();

            if !done {
                return Poll::Pending;
            }
            let futures = unsafe { futures.as_mut().get_unchecked_mut() };

            Poll::Ready((
                futures.0.take_output().unwrap(),
                futures.1.take_output().unwrap(),
            ))
        })
        .await
    }
}

enum MaybeDone<F: Future> {
    Future(F),
    Done(F::Output),
    Taken,
}

impl<F: Future> MaybeDone<F> {
    pub fn take_output(&mut self) -> Option<F::Output> {
        match *self {
            MaybeDone::Done(_) => match mem::replace(self, Self::Taken) {
                MaybeDone::Done(val) => Some(val),
                _ => unreachable!(),
            },
            _ => None,
        }
    }
}

impl<F: Future> Future for MaybeDone<F> {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        unsafe {
            match *self.as_mut().get_unchecked_mut() {
                MaybeDone::Future(ref mut f) => match Pin::new_unchecked(f).poll(cx) {
                    Poll::Ready(val) => self.set(Self::Done(val)),
                    Poll::Pending => return Poll::Pending,
                },
                MaybeDone::Done(_) => {}
                MaybeDone::Taken => unreachable!(),
            }
        }

        Poll::Ready(())
    }
}
