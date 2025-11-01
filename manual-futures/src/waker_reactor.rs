use mio::{Events, Interest, Registry, Token};
use std::{
    sync::{Mutex, OnceLock},
    task::Waker,
};

pub static REGISTRY: OnceLock<Mutex<Registry>> = OnceLock::new();
static WAKER: OnceLock<Mutex<Option<Waker>>> = OnceLock::new();

pub fn initialize_reactor() -> (mio::Poll, Events) {
    let epoll = mio::Poll::new().unwrap();
    REGISTRY
        .set(Mutex::new(epoll.registry().try_clone().unwrap()))
        .unwrap();
    WAKER.set(Mutex::new(None)).unwrap();
    (epoll, Events::with_capacity(10))
}

pub fn register(source: &mut impl mio::event::Source, interest: Interest, waker: Waker) {
    let waker_lock = WAKER.get().unwrap();
    let mut waker_guard = waker_lock.lock().unwrap();
    let is_reregister = waker_guard.is_some();
    *waker_guard = Some(waker);
    drop(waker_guard);

    let registry = REGISTRY.get().unwrap().lock().unwrap();
    if is_reregister {
        registry.reregister(source, Token(0), interest).unwrap();
    } else {
        registry.register(source, Token(0), interest).unwrap();
    }
}

pub fn run_reactor() {
    let (mut epoll, mut events) = initialize_reactor();

    loop {
        epoll.poll(&mut events, None).unwrap();

        if !events.is_empty() {
            let waker_lock = WAKER.get().unwrap();
            if let Some(waker) = waker_lock.lock().unwrap().as_ref() {
                waker.wake_by_ref();
            }
        }
    }
}
