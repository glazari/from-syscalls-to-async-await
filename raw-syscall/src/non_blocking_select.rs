use crate::sys_libc::{self, FdSet, SocketFd};
use crate::{cyan, green};
use std::{mem, ptr};

pub fn non_blocking_calls() -> Result<(), anyhow::Error> {
    let start = std::time::Instant::now();
    green!("--- non_blocking_select ---");
    // make request using non-blocking socket
    let mut sockets = vec![];
    // created sockets
    for i in 0..3 {
        let sock = sys_libc::create_non_blocking_tcp_socket()?;
        sockets.push(sock);
    }

    let server_addr = sys_libc::create_ipv4_sockaddr("127.0.0.1", 3000)?;

    // connect all sockets
    for socket in sockets.iter() {
        sys_libc::connect(socket, &server_addr)?;
    }

    // wait until all are connected using select
    wait_for_connections(&sockets)?;

    // send data to all sockets
    let request = "GET / HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n";
    for socket in sockets.iter() {
        sys_libc::send(socket, request.as_bytes())?;
    }

    // receive data from all sockets using non-blocking select
    let responses = receive_all_non_blocking(&sockets)?;

    // print all responses
    for (i, response) in responses.iter().enumerate() {
        println!("Response {}:\n{}", i, String::from_utf8_lossy(response));
    }
    println!("Non-blocking select duration: {:?}", start.elapsed());
    Ok(())
}

pub fn receive_all_non_blocking(sockets: &[SocketFd]) -> Result<Vec<Vec<u8>>, anyhow::Error> {
    let mut responses = vec![Vec::new(); sockets.len()];
    let mut finished = vec![false; sockets.len()];
    let mut fd_set = FdSet::new();
    let max_fd = sockets.iter().map(|s| s.0).max().unwrap();

    while !finished.iter().all(|&f| f) {
        fd_set.clear();
        let mut has_active_sockets = false;

        for (i, socket) in sockets.iter().enumerate() {
            if !finished[i] {
                fd_set.set(&socket);
                has_active_sockets = true;
            }
        }

        if !has_active_sockets {
            break;
        }
        let result = sys_libc::select_read(max_fd + 1, &mut fd_set, None)?;

        if result > 0 {
            for (i, socket) in sockets.iter().enumerate() {
                if !finished[i] && fd_set.is_set(&socket) {
                    let mut temp_buf = [0u8; 4096];
                    match sys_libc::recv(socket, &mut temp_buf)? {
                        Some(0) => {
                            finished[i] = true;
                            crate::cyan!("Socket {} finished receiving", socket);
                        }
                        Some(bytes_received) => {
                            responses[i].extend_from_slice(&temp_buf[..bytes_received]);
                        }
                        None => {} // Would block, continue
                    }
                }
            }
        }
    }

    for (i, response) in responses.iter().enumerate() {
        crate::cyan!("Total bytes received from socket {}: {}", i, response.len());
    }

    Ok(responses)
}

pub fn wait_for_connections(sockets: &[SocketFd]) -> Result<(), anyhow::Error> {
    let mut fd_set = sys_libc::FdSet::new();
    let max_fd = sockets.iter().map(|s| s.0).max().unwrap();

    loop {
        fd_set.clear();
        for socket in sockets {
            fd_set.set(socket);
        }

        let result = sys_libc::select_write(max_fd + 1, &mut fd_set, None)?;

        if result > 0 {
            let mut all_connected = true;
            for socket in sockets {
                if fd_set.is_set(socket) {
                    // Check if connection completed successfully
                    let error = sys_libc::get_socket_error(socket)?;
                    if error != 0 {
                        return Err(anyhow::anyhow!("Connection failed with error: {}", error));
                    }
                    crate::cyan!("Socket {} connected successfully", socket);
                } else {
                    all_connected = false;
                }
            }
            if all_connected {
                crate::cyan!("All sockets connected!");
                break;
            }
        }
    }
    Ok(())
}
