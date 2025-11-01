use crate::{cyan, green};
use std::io::{Read, Write};
use std::mem;
use std::net::TcpStream;
use std::os::unix::io::AsRawFd;

use crate::sys_libc::libc;
use crate::sys_libc::libc::epoll_event;
use crate::sys_libc::{self, EpollEvent, EpollFd, SocketFd};

pub fn non_blocking_call() -> Result<(), anyhow::Error> {
    let start = std::time::Instant::now();
    green!("--- non_blocking_std ---");

    let epoll_fd = sys_libc::epoll_create1(0)?;
    let IN_AND_OUT_EDGE_TRIGGER = (libc::EPOLLIN | libc::EPOLLOUT | libc::EPOLLET) as u32;

    let mut streams = vec![];
    let mut sockets = vec![];
    for i in 0..3 {
        let stream = create_non_blocking_stream("127.0.0.1:3000")?;
        sockets.push(SocketFd(stream.as_raw_fd()));
        streams.push(stream);

        let socket = &sockets[i];
        let mut event = EpollEvent::new(&socket, IN_AND_OUT_EDGE_TRIGGER as u32);
        sys_libc::epoll_ctl(&epoll_fd, libc::EPOLL_CTL_ADD, socket, &mut event)?;
    }

    let request = "GET / HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n";
    let mut events: [epoll_event; 10] = unsafe { std::mem::zeroed() };
    send_all(&sockets, &mut streams, request, &epoll_fd, &mut events)?;
    println!("Sent all requests, now reading responses...");
    let responses = read_all_non_blocking(&mut streams, &sockets, &epoll_fd, &mut events)?;
    println!("Received all responses:");
    for (i, response) in responses.iter().enumerate() {
        println!("Response {}:\n{}", i, String::from_utf8_lossy(response));
    }

    // sockets are actually managed by TcpStream, so we need to forget them here
    // to avoid double closing
    mem::forget(sockets);

    println!("Non-blocking std duration: {:?}", start.elapsed());

    Ok(())
}

pub fn create_non_blocking_stream(addr: &str) -> Result<TcpStream, anyhow::Error> {
    let stream = TcpStream::connect(addr)?;
    stream.set_nonblocking(true)?;
    stream.set_nodelay(true)?; // disable Nagle's algorithm
    Ok(stream)
}

fn read_all_non_blocking(
    streams: &mut [TcpStream],
    sockets: &[SocketFd],
    epoll_fd: &EpollFd,
    events: &mut [epoll_event],
) -> Result<Vec<Vec<u8>>, anyhow::Error> {
    let timeout = 5000; // 5 seconds

    let mut responses = vec![Vec::new(); sockets.len()];
    let mut finished = vec![false; sockets.len()];

    // try to receive from all of them first (since we use edge-triggered)
    for (idx, stream) in streams.iter_mut().enumerate() {
        finished[idx] = read_until_would_block(stream, &mut responses[idx])?;
    }
    while !finished.iter().all(|&f| f) {
        let nfds = sys_libc::epoll_wait(epoll_fd, events, timeout)?;
        if nfds == 0 {
            return Err(anyhow::anyhow!("Timeout waiting for data"));
        }
        for i in 0..nfds as usize {
            let event = &events[i];
            let fd = unsafe { event.data.u64 as i32 };
            if let Some((idx, _)) = sockets.iter().enumerate().find(|(_, s)| s.0 == fd) {
                if finished[idx] {
                    continue; // already finished
                }
                let stream = &mut streams[idx];
                if event.events
                    & (libc::EPOLLERR as u32 | libc::EPOLLHUP as u32 | libc::POLLNVAL as u32)
                    != 0
                {
                    finished[idx] = read_until_would_block(stream, &mut responses[idx])?;
                    sys_libc::epoll_ctl_remove(epoll_fd, &sockets[idx])?;
                }

                if event.events & (libc::EPOLLIN as u32) != 0 {
                    finished[idx] = read_until_would_block(stream, &mut responses[idx])?;
                    if finished[idx] {
                        sys_libc::epoll_ctl_remove(epoll_fd, &sockets[idx])?;
                    }
                }
            }
        }
    }

    Ok(responses)
}

fn read_until_would_block(
    stream: &mut TcpStream,
    buffer: &mut Vec<u8>,
) -> Result<bool, anyhow::Error> {
    let mut temp_buf = [0u8; 4096];
    loop {
        match stream.read(&mut temp_buf) {
            Ok(0) => {
                // Connection closed
                return Ok(true);
            }
            Ok(n) => {
                buffer.extend_from_slice(&temp_buf[..n]);
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                // No more data to read
                return Ok(false);
            }
            Err(e) => {
                return Err(anyhow::anyhow!("Error reading from socket: {}", e));
            }
        }
    }
}

pub fn send_all(
    sockets: &[SocketFd],
    streams: &mut [TcpStream],
    request: &str,
    epoll_fd: &EpollFd,
    events: &mut [epoll_event],
) -> Result<(), anyhow::Error> {
    let mut sent_requests = vec![false; sockets.len()];
    for (i, socket) in sockets.iter().enumerate() {
        let stream = &mut streams[i];
        match stream.write(request.as_bytes()) {
            Ok(n) if n == request.len() => {
                sent_requests[i] = true;
                cyan!("Sent full request on socket {}", socket.0);
            }
            Ok(n) => {
                cyan!(
                    "Partial write on socket {}: sent {}/{} bytes",
                    socket.0,
                    n,
                    request.len()
                );
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                cyan!("Socket {} would block on write, will retry later", socket.0);
            }
            Err(e) => {
                return Err(anyhow::anyhow!(
                    "Error writing to socket {}: {}",
                    socket.0,
                    e
                ));
            }
        }
    }

    while !sent_requests.iter().all(|&s| s) {
        let nfds = sys_libc::epoll_wait(epoll_fd, events, 5000)?; // 5 second timeout
        if nfds == 0 {
            return Err(anyhow::anyhow!(
                "Timeout waiting for sockets to be writable"
            ));
        }
        for i in 0..nfds as usize {
            let event = &events[i];
            let fd = unsafe { event.data.u64 as i32 };
            if let Some((idx, _)) = sockets.iter().enumerate().find(|(_, s)| s.0 == fd) {
                if sent_requests[idx] {
                    continue; // already sent
                }
                let stream = &mut streams[idx];
                match stream.write(request.as_bytes()) {
                    Ok(n) if n == request.len() => {
                        sent_requests[idx] = true;
                        cyan!("Sent full request on socket {}", fd);
                    }
                    Ok(n) => {
                        cyan!(
                            "Partial write on socket {}: sent {}/{} bytes",
                            fd,
                            n,
                            request.len()
                        );
                    }
                    Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                        cyan!("Socket {} would block on write, will retry later", fd);
                    }
                    Err(e) => {
                        return Err(anyhow::anyhow!("Error writing to socket {}: {}", fd, e));
                    }
                }
            }
        }
    }

    Ok(())
}
