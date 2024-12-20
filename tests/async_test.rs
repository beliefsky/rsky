#[cfg(test)]
mod tests {
    use futures::executor::block_on;
    use std::{
        future::Future,
        sync::{Arc, Mutex},
        thread,
    };

    #[test]
    fn async_test() {
        let share_state = Arc::new(Mutex::new(ShareData::new()));

        let thread_share_state = Arc::clone(&share_state);
        thread::spawn(move || {
            // 创建一个线程，让回调延迟10秒
            for i in 0..10 {
                println!("loop {}", i);
                let mut state = thread_share_state.lock().unwrap();
                state.count += 1;
                if let Some(waker) = state.waker.take() {
                    waker.wake_by_ref();
                }
            }
        });

        let result = block_on(async_func(share_state));

        println!("==========> {}", result);
    }

    async fn async_func(state: Arc<Mutex<ShareData>>) -> u32 {
        Reader::new(state).await
    }

    struct ShareData {
        count: u32,
        waker: Option<std::task::Waker>,
    }

    impl ShareData {
        pub(crate) fn new() -> Self {
            ShareData {
                count: 0,
                waker: None,
            }
        }
    }

    struct Reader {
        state: Arc<Mutex<ShareData>>,
    }

    impl Reader {
        pub(crate) fn new(data: Arc<Mutex<ShareData>>) -> Self {
            Self { state: data }
        }
    }

    impl Future for Reader {
        type Output = u32;

        fn poll(
            self: std::pin::Pin<&mut Self>,
            ctx: &mut std::task::Context<'_>,
        ) -> std::task::Poll<Self::Output> {
            let mut state = self.state.lock().unwrap();
            if state.count == 0 {
                if state.waker.is_none() {
                    ctx.waker().wake_by_ref();
                    state.waker = Some(ctx.waker().clone());
                }
                std::task::Poll::Pending
            } else {
                std::task::Poll::Ready(state.count)
            }
        }
    }
}
