use mio::net::TcpStream;
use std::{
    future::Future,
    io::Read,
    pin::Pin,
    task::{Context, Poll},
};

const READABLE: mio::Interest = mio::Interest::READABLE;
use crate::waker_reactor;

pub fn receive_async<'a>(stream: &'a mut TcpStream) -> ReceiveFuture<'a> {
    ReceiveFuture {
        stream,
        response: Vec::new(),
        state: ReceiveState::Receiving,
    }
}

pub struct ReceiveFuture<'a> {
    stream: &'a mut TcpStream,
    response: Vec<u8>,
    state: ReceiveState,
}

pub enum ReceiveState {
    Receiving,
    Done,
}

impl<'a> Future for ReceiveFuture<'a> {
    type Output = Vec<u8>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Vec<u8>> {
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
                        waker_reactor::register(this.stream, READABLE, cx.waker().clone());
                        Poll::Pending
                    }
                    Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                        waker_reactor::register(this.stream, READABLE, cx.waker().clone());
                        Poll::Pending
                    }
                    Err(e) => panic!("Read failed {}", e),
                }
            }
            ReceiveState::Done => Poll::Ready(this.response.clone()),
        }
    }
}
