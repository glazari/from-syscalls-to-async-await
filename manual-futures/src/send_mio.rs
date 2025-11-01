use mio::net::TcpStream;
use std::{
    future::Future,
    io::Write,
    pin::Pin,
    task::{Context, Poll},
};

use crate::epoll_executor::REGISTRY;

pub fn send_async<'a>(stream: &'a mut TcpStream, request: &str) -> SendFuture<'a> {
    SendFuture {
        stream: stream,
        request: request.to_string(),
        sent: 0,
        state: SendState::Sending,
    }
}

pub struct SendFuture<'a> {
    stream: &'a mut TcpStream,
    request: String,
    sent: usize,
    state: SendState,
}

enum SendState {
    Sending,
    Done,
}

impl<'a> Future for SendFuture<'a> {
    type Output = usize;

    fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<usize> {
        let this = self.get_mut();
        match this.state {
            SendState::Sending => {
                let bytes = this.request.as_bytes();
                match this.stream.write(bytes[this.sent..].as_ref()) {
                    Ok(n) => {
                        this.state = SendState::Done;
                        this.sent += n;
                        if this.sent >= bytes.len() {
                            Poll::Ready(this.sent)
                        } else {
                            let registry = REGISTRY.get().unwrap().lock().unwrap();
                            registry
                                .reregister(this.stream, mio::Token(0), mio::Interest::WRITABLE)
                                .expect("Failed to reregister stream");
                            Poll::Pending
                        }
                    }
                    Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => Poll::Pending,
                    Err(e) => panic!("Write failed {}", e),
                }
            }
            SendState::Done => Poll::Ready(this.sent),
        }
    }
}
