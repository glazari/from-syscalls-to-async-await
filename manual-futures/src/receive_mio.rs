use mio::net::TcpStream;
use std::{
    future::Future,
    io::Read,
    pin::Pin,
    task::{Context, Poll},
};

use crate::epoll_executor::REGISTRY;

pub fn receive_async<'a>(stream: &'a mut TcpStream) -> ReceiveFuture<'a> {
    ReceiveFuture {
        stream: stream,
        response: Vec::new(),
        state: ReceiveState::Receiving,
        registered: false,
    }
}

pub struct ReceiveFuture<'a> {
    stream: &'a mut TcpStream,
    response: Vec<u8>,
    state: ReceiveState,
    registered: bool,
}

pub enum ReceiveState {
    Receiving,
    Done,
}

impl<'a> Future for ReceiveFuture<'a> {
    type Output = Vec<u8>;

    fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Vec<u8>> {
        let this = self.get_mut();
        match this.state {
            ReceiveState::Receiving => {
                let mut buf = [0; 1024];
                match this.stream.read(&mut buf) {
                    Ok(0) => {
                        this.state = ReceiveState::Done;
                        Poll::Ready(this.response.clone())
                    }
                    Ok(n) => {
                        this.response.extend_from_slice(&buf[..n]);
                        if !this.registered {
                            let registry = REGISTRY.get().unwrap().lock().unwrap();
                            registry
                                .reregister(this.stream, mio::Token(0), mio::Interest::READABLE)
                                .expect("Failed to register stream");
                            this.registered = true;
                        }

                        Poll::Pending
                    }
                    Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => Poll::Pending,
                    Err(e) => panic!("Read failed {}", e),
                }
            }
            ReceiveState::Done => Poll::Ready(this.response.clone()),
        }
    }
}
