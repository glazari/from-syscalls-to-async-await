use std::{
    future::Future,
    net::TcpStream,
    pin::Pin,
    task::{Context, Poll},
};

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

    fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
        let stream = TcpStream::connect(&self.address).unwrap();
        stream.set_nonblocking(true).unwrap();
        Poll::Ready(stream)
    }
}
