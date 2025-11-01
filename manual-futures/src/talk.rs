use std::io::{ErrorKind::WouldBlock, Read};
use std::net::TcpStream;
use std::pin::Pin;
use std::task::{Context, Poll};

impl<'a> Future for ReceiveFut<'a> {
  type Output = Vec<u8>;

  fn poll(
    self: Pin<&mut Self>,
    cx: &mut Context<'_>,
  ) -> Poll<Vec<u8>> {
    let this = self.get_mut();
    match this.state {
      Receiving => _rec(this),
      Done => Poll::Ready(this.response.clone()),
    }
  }
}

use std::task::Poll::{Pending, Ready};

pub fn block_on<T>(
  mut f: impl Future<Output = T>,
) -> T {
  let parker = Arc::new(Parker::new());
  let waker = Waker::from(parker.clone());
  let mut cx = Context::from_waker(&waker);
  let mut f = unsafe { Pin::new_unchecked(&mut f) };

  loop {
    match f.as_mut().poll(&mut cx) {
      Poll::Ready(out) => break out,
      Poll::Pending => parker.park(),
    }
  }
}

pub struct ReceiveFut<'a> {
  stream: &'a mut TcpStream,
  response: Vec<u8>,
  state: State,
  registered: bool,
}

pub enum State {
  Receiving,
  Done,
}

pub fn receive_async<'a>(
  stream: &mut TcpStream,
) -> ReceiveFut<'a> {
  ReceiveFut {
    stream: stream,
    response: Vec::new(),
    state: State::Receiving,
    registered: false,
  }
}

use mio::Interest::READABLE;
use std::task::Poll::{Pending, Ready};

fn _rec(fut: &mut ReceiveFut) -> Poll<Vec<u8>> {
  let mut buf = [0; 1024];
  match fut.stream.read(&mut buf) {
    // ...igual antes
    Err(e) if e.kind() == WouldBlock => {
      let registry = get_registry();

      let token = mio::Token(0);
      registry
        .reregister(fut.stream, token, READABLE)
        .unwrap();
      Poll::Pending
    }
    Err(e) => panic!("Read failed {}", e),
  }
}

pub struct Parker { 
 woken: Mutex<bool>, condvar: Condvar,
}
impl Parker {
 pub fn park(&self) {
  let mut woken = self.woken.lock().unwrap();
  while !*woken {
   woken = self.condvar.wait(woken).unwrap();
  }
  *woken = false;
 }
 fn unpark(&self) {
  let mut woken = self.woken.lock().unwrap();
  *woken = true;
  self.condvar.notify_one();
 }
}
impl Wake for Parker {
 fn wake(self: Arc<Self>) { self.unpark() }
}
