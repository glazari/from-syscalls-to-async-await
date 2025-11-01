use std::{
    future::Future,
    io::Write,
    net::TcpStream,
    pin::Pin,
    task::{Context, Poll},
};

pub fn send_async(stream: &mut TcpStream, request: &str) -> SendFuture {
    SendFuture {
        stream: stream.try_clone().unwrap(),
        request: request.to_string(),
        sent: 0,
        state: SendState::Sending,
    }
}

pub struct SendFuture {
    stream: TcpStream,
    request: String,
    sent: usize,
    state: SendState,
}

enum SendState {
    Sending,
    Done,
}

impl Future for SendFuture {
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
