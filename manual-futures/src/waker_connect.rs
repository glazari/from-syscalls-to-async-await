use mio::net::TcpStream;
use std::{
    future::Future,
    net::SocketAddr,
    pin::Pin,
    task::{Context, Poll},
};

use crate::waker_reactor;

pub fn connect_async(address: &str) -> ConnectFuture {
    ConnectFuture {
        address: address.to_string(),
    }
}

pub struct ConnectFuture {
    address: String,
}

impl Future for ConnectFuture {
    type Output = TcpStream;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let socket_addr: SocketAddr = self.address.parse().unwrap();
        let mut stream = TcpStream::connect(socket_addr).unwrap();
        waker_reactor::register(&mut stream, mio::Interest::READABLE, cx.waker().clone());
        Poll::Ready(stream)
    }
}
