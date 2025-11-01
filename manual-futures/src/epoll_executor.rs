use mio::{Events, Registry};
use std::{
    pin::Pin,
    sync::{Mutex, OnceLock},
    task::{Context, Poll},
};

pub static REGISTRY: OnceLock<Mutex<Registry>> = OnceLock::new();

pub fn initialize_registry() -> (mio::Poll, Events) {
    let epoll = mio::Poll::new().expect("Failed to create mio::Poll");
    REGISTRY
        .set(Mutex::new(epoll.registry().try_clone().unwrap()))
        .unwrap();
    (epoll, Events::with_capacity(10))
}

pub fn block_on<T>(mut f: impl Future<Output = T>) -> T {
    let (mut epoll, mut events) = initialize_registry();
    let waker = futures::task::noop_waker();
    let mut cx = Context::from_waker(&waker);
    let mut f = unsafe { Pin::new_unchecked(&mut f) };

    println!("Starting epoll_executor::block_on");
    let mut tries = 0;
    loop {
        println!("  epoll_executor::block_on try {}", tries);
        match f.as_mut().poll(&mut cx) {
            Poll::Ready(out) => break out,
            Poll::Pending => epoll.poll(&mut events, None).unwrap(),
        }
        tries += 1;
    }
}
