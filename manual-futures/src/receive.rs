use std::{
    future::Future,
    io::Read,
    net::TcpStream,
    pin::Pin,
    task::{Context, Poll},
};

pub fn receive_async(stream: &mut TcpStream) -> ReceiveFuture {
    ReceiveFuture {
        stream: stream.try_clone().unwrap(),
        response: Vec::new(),
        state: ReceiveState::Receiving,
    }
}

pub struct ReceiveFuture {
    stream: TcpStream,
    response: Vec<u8>,
    state: ReceiveState,
}

pub enum ReceiveState {
    Receiving,
    Done,
}

impl Future for ReceiveFuture {
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
                        println!("Waker {:#?}", cx.waker());
                        cx.waker().wake_by_ref();
                        Poll::Pending
                    }
                    Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                        cx.waker().wake_by_ref();
                        Poll::Pending
                    }
                    Err(e) => panic!("Read failed {}", e),
                }
            }
            ReceiveState::Done => Poll::Ready(this.response.clone()),
        }
    }
}
