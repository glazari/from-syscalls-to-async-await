use std::{
    pin::Pin,
    task::{Context, Poll},
    thread::sleep,
    time::Duration,
};

pub fn block_on<T>(mut f: impl Future<Output = T>) -> T {
    let waker = futures::task::noop_waker();
    let mut cx = Context::from_waker(&waker);
    let mut f = unsafe { Pin::new_unchecked(&mut f) };

    println!("Starting executor_naive::block_on");
    let mut tries = 0;
    loop {
        println!("  executor_naive::block_on try {}", tries);
        match f.as_mut().poll(&mut cx) {
            Poll::Ready(out) => break out,
            Poll::Pending => sleep(Duration::from_millis(20)),
        }
        tries += 1;
    }
}
