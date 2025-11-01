use mio::net::TcpStream;
use std::{
    future::Future,
    net::SocketAddr,
    pin::Pin,
    task::{Context, Poll},
};

use crate::epoll_executor::REGISTRY;

pub fn connect_async_mio(address: &str) -> ConnectFuture {
    ConnectFuture {
        address: address.to_string(),
    }
}

pub struct ConnectFuture {
    address: String,
}

impl Future for ConnectFuture {
    type Output = TcpStream;

    fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
        let socket_addr: SocketAddr = self.address.parse().unwrap();
        let mut stream = TcpStream::connect(socket_addr).unwrap();
        let registry = REGISTRY.get().unwrap().lock().unwrap();
        registry
            .register(&mut stream, mio::Token(0), mio::Interest::READABLE)
            .expect("Failed to register stream");
        Poll::Ready(stream)
    }
}
