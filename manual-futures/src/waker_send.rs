use mio::net::TcpStream;
use std::{
    future::Future,
    io::Write,
    pin::Pin,
    task::{Context, Poll},
};

use crate::waker_reactor;

const WRITABLE: mio::Interest = mio::Interest::WRITABLE;
pub fn send_async<'a>(stream: &'a mut TcpStream, request: &str) -> SendFuture<'a> {
    SendFuture {
        stream,
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

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<usize> {
        let this = self.get_mut();
        match this.state {
            SendState::Sending => {
                let bytes = this.request.as_bytes();
                match this.stream.write(bytes[this.sent..].as_ref()) {
                    Ok(n) => {
                        this.sent += n;
                        if this.sent >= bytes.len() {
                            this.state = SendState::Done;
                            Poll::Ready(this.sent)
                        } else {
                            waker_reactor::register(this.stream, WRITABLE, cx.waker().clone());
                            Poll::Pending
                        }
                    }
                    Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                        waker_reactor::register(this.stream, WRITABLE, cx.waker().clone());
                        Poll::Pending
                    }
                    Err(e) => panic!("Write failed {}", e),
                }
            }
            SendState::Done => Poll::Ready(this.sent),
        }
    }
}
