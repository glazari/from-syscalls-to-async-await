use crate::{cyan, green};
use crate::sys_libc::libc::{POLLERR, POLLHUP, POLLIN, POLLNVAL, POLLOUT};
use crate::sys_libc::{self, PollFd, SocketFd};

pub fn non_blocking_calls() -> Result<(), anyhow::Error> {
    let start = std::time::Instant::now();
    green!("--- non_blocking_poll ---");
    let mut sockets = vec![];
    for _ in 0..3 {
        let socket = sys_libc::create_non_blocking_tcp_socket()?;
        sockets.push(socket);
    }

    let server_addr = sys_libc::create_ipv4_sockaddr("127.0.0.1", 3000)?;

    for socket in sockets.iter() {
        sys_libc::connect(socket, &server_addr)?;
    }
    let mut poll_fds = sockets
        .iter()
        .map(|socket| PollFd::new(socket, POLLOUT))
        .collect::<Vec<_>>();

    wait_for_connections(&mut poll_fds)?;

    let request = "GET / HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n";
    for socket in sockets.iter() {
        sys_libc::send(socket, request.as_bytes())?;
    }

    // receive data from all sockets using non-blocking poll
    let responses = receive_all_non_blocking(&mut poll_fds, &sockets)?;

    for (i, response) in responses.iter().enumerate() {
        println!("Response {}:\n{}", i, String::from_utf8_lossy(response));
    }

    println!("Non-blocking poll duration: {:?}", start.elapsed());
    Ok(())
}

pub fn receive_all_non_blocking(
    poll_fds: &mut [PollFd],
    sockets: &[SocketFd],
) -> Result<Vec<Vec<u8>>, anyhow::Error> {
    let timeout = 5000; // 5 seconds
    // reset revents before polling
    poll_fds.iter_mut().for_each(PollFd::reset_revents);
    poll_fds.iter_mut().for_each(|pfd| pfd.set_events(POLLIN));

    let mut responses = vec![Vec::new(); poll_fds.len()];
    let mut finished = vec![false; poll_fds.len()];

    while !finished.iter().all(|&f| f) {
        let poll_result = sys_libc::poll(poll_fds, timeout)?;
        if poll_result == 0 {
            return Err(anyhow::anyhow!("Timeout waiting for data"));
        }
        // check for errors
        let errors: Vec<_> = poll_fds
            .iter()
            .filter(|pfd| pfd.revents() & (POLLERR | POLLHUP | POLLNVAL) != 0)
            .collect();
        if !errors.is_empty() {
            return Err(anyhow::anyhow!("Error on sockets: {:?}", errors));
        }

        for (i, pfd) in poll_fds.iter_mut().enumerate() {
            if finished[i] {
                continue;
            }
            if pfd.revents() & POLLIN != 0 {
                let mut buf = [0u8; 4096];
                let socket = &sockets[i];
                match sys_libc::recv(socket, &mut buf)? {
                    Some(0) => {
                        finished[i] = true; // connection closed
                    }
                    Some(n) => {
                        responses[i].extend_from_slice(&buf[..n]);
                    }
                    None => {} // would block continue
                }
            }
        }
        // reset revents for next poll
        poll_fds.iter_mut().for_each(PollFd::reset_revents);
    }

    Ok(responses)
}

pub fn wait_for_connections(poll_fds: &mut [PollFd]) -> Result<(), anyhow::Error> {
    let timeout = 5000; // 5 seconds
    // reset revents before polling
    poll_fds.iter_mut().for_each(PollFd::reset_revents);

    while !poll_fds.iter().all(|pfd| pfd.revents() & POLLOUT != 0) {
        let poll_result = sys_libc::poll(poll_fds, timeout)?;
        if poll_result == 0 {
            return Err(anyhow::anyhow!("Timeout waiting for connections"));
        }
        // check for errors
        let errors: Vec<_> = poll_fds
            .iter()
            .filter(|pfd| pfd.revents() & (POLLERR | POLLHUP | POLLNVAL) != 0)
            .collect();
        if !errors.is_empty() {
            return Err(anyhow::anyhow!("Error on sockets: {:?}", errors));
        }
    }

    Ok(())
}
