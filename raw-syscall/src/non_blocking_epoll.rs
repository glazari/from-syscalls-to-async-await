use crate::sys_libc::libc;
use crate::sys_libc::libc::epoll_event;
use crate::sys_libc::{self, EpollEvent, EpollFd, SocketFd};
use crate::{cyan, green};

pub fn non_blocking_calls() -> Result<(), anyhow::Error> {
    let start = std::time::Instant::now();
    green!("--- non_blocking_epoll ---");
    let epoll_fd = sys_libc::epoll_create1(0)?;

    let mut events: [epoll_event; 10] = unsafe { std::mem::zeroed() };

    let mut sockets = vec![];
    let IN_AND_OUT_EDGE_TRIGGER = (libc::EPOLLIN | libc::EPOLLOUT | libc::EPOLLET) as u32;
    for _ in 0..3 {
        let socket = sys_libc::create_non_blocking_tcp_socket()?;

        let mut event = EpollEvent::new(&socket, IN_AND_OUT_EDGE_TRIGGER as u32);
        sys_libc::epoll_ctl(&epoll_fd, libc::EPOLL_CTL_ADD, &socket, &mut event)?;
        sockets.push(socket);
    }

    let server_addr = sys_libc::create_ipv4_sockaddr("127.0.0.1", 3000)?;
    for socket in &sockets {
        sys_libc::connect(socket, &server_addr)?;
    }

    let request = "GET / HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n";

    send_all(&sockets, request, &epoll_fd, &mut events)?;
    let responses = receive_all_non_blocking(&sockets, &epoll_fd, &mut events)?;
    for (i, response) in responses.iter().enumerate() {
        println!("Response {}:\n{}", i, String::from_utf8_lossy(response));
    }

    println!("Non-blocking epoll duration: {:?}", start.elapsed());
    Ok(())
}

fn receive_all_non_blocking(
    sockets: &[SocketFd],
    epoll_fd: &EpollFd,
    events: &mut [epoll_event],
) -> Result<Vec<Vec<u8>>, anyhow::Error> {
    let timeout = 5000; // 5 seconds

    let mut responses = vec![Vec::new(); sockets.len()];
    let mut finished = vec![false; sockets.len()];

    // try to receive from all of them first (since we use edge-triggered)
    for (idx, socket) in sockets.iter().enumerate() {
        finished[idx] = read_until_would_block(socket, &mut responses[idx])?;
    }
    while !finished.iter().all(|&f| f) {
        let nfds = sys_libc::epoll_wait(epoll_fd, events, timeout)?;
        if nfds == 0 {
            return Err(anyhow::anyhow!("Timeout waiting for data"));
        }
        for i in 0..nfds as usize {
            let event = &events[i];
            let events = event.events;
            let fd = unsafe { event.data.u64 as i32 };
            println!("Epoll event: fd={}, events={:#b}", fd, events);
            let idx = sockets.iter().position(|s| s.0 == fd).unwrap();
            if event.events & (libc::EPOLLHUP | libc::EPOLLERR as u32) != 0 {
                println!("Socket {} closed or error, draining socket", sockets[idx]);
                finished[idx] = true;
                let f = read_until_would_block(&sockets[idx], &mut responses[idx])?;
                sys_libc::epoll_ctl_remove(epoll_fd, &sockets[idx])?;
            }

            if (event.events & libc::EPOLLIN as u32) != 0 && !finished[idx] {
                finished[idx] = read_until_would_block(&sockets[idx], &mut responses[idx])?;
            }
        }
    }

    Ok(responses)
}

fn read_until_would_block(socket: &SocketFd, buffer: &mut Vec<u8>) -> Result<bool, anyhow::Error> {
    let mut temp_buf = [0u8; 4096];
    let finished = loop {
        let bytes_received = sys_libc::recv(socket, &mut temp_buf)?;
        match bytes_received {
            None => {
                break false; // would block, try again later
            }
            Some(0) => {
                break true; // finished receiving
            }
            Some(n) => {
                buffer.extend_from_slice(&temp_buf[..n]);
            }
        }
    };
    Ok(finished)
}

fn send_all(
    sockets: &[SocketFd],
    request: &str,
    epoll_fd: &EpollFd,
    events: &mut [epoll_event],
) -> Result<(), anyhow::Error> {
    let mut sent_requests = vec![false; sockets.len()];
    for (idx, socket) in sockets.iter().enumerate() {
        let ret = sys_libc::send(socket, request.as_bytes())?;
        match ret {
            Some(n) if n == request.len() => {
                sent_requests[idx] = true;
            }
            Some(n) => {
                sent_requests[idx] = true; // for simplicity, mark as sent
                cyan!("Partial send of {} bytes", n);
            }
            None => {
                // would block, try again later
            }
        }
    }

    while !sent_requests.iter().all(|&s| s) {
        let nfds = sys_libc::epoll_wait(&epoll_fd, events, 5000)?;
        if nfds == 0 {
            return Err(anyhow::anyhow!("Timeout waiting for events"));
        }
        for i in 0..nfds as usize {
            let event = &events[i];
            let fd = unsafe { event.data.u64 as i32 };
            let idx = sockets.iter().position(|s| s.0 == fd).unwrap();
            if (event.events & libc::EPOLLOUT as u32) != 0 && !sent_requests[idx] {
                let ret = sys_libc::send(&sockets[idx], request.as_bytes())?;
                match ret {
                    Some(n) if n == request.len() => {
                        sent_requests[idx] = true;
                    }
                    Some(n) => {
                        sent_requests[idx] = true; // for simplicity, mark as sent
                        cyan!("Partial send of {} bytes", n);
                    }
                    None => {
                        // would block, try again later
                    }
                }
            }
        }
    }

    Ok(())
}
