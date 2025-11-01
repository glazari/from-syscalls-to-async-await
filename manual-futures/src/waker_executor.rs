use crate::waker::Parker;
use std::{
    pin::Pin,
    sync::Arc,
    task::{Context, Poll, Waker},
};

pub fn block_on<T>(mut f: impl Future<Output = T>) -> T {
    let parker = Arc::new(Parker::new());
    let waker = Waker::from(parker.clone());
    let mut cx = Context::from_waker(&waker);
    let mut f = unsafe { Pin::new_unchecked(&mut f) };

    println!("Starting waker_executor::block_on");
    let mut tries = 0;
    loop {
        println!("  waker_executor::block_on try {}", tries);
        match f.as_mut().poll(&mut cx) {
            Poll::Ready(out) => break out,
            Poll::Pending => parker.park(),
        }
        tries += 1;
    }
}
