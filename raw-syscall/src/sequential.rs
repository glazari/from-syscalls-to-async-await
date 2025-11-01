use crate::{
    green,
    sys_libc::{self, SocketFd},
};

pub fn sequential_calls() -> Result<(), anyhow::Error> {
    let start = std::time::Instant::now();
    green!("--- sequential ---");
    let request = "GET / HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n";

    // run 3 times
    for i in 0..3 {
        let resp = make_request("127.0.0.1", 3000, request)?;
        println!("Response:\n{}", String::from_utf8_lossy(&resp));
    }

    println!("Sequential duration: {:?}", start.elapsed());
    Ok(())
}

pub fn make_request(addr: &str, port: u16, request: &str) -> Result<Vec<u8>, anyhow::Error> {
    let sockfd = sys_libc::create_tcp_socket()?;
    let server_addr = sys_libc::create_ipv4_sockaddr(addr, port)?;
    sys_libc::connect(&sockfd, &server_addr)?;
    sys_libc::send(&sockfd, request.as_bytes())?;
    let response = receive_all(&sockfd)?;
    Ok(response)
}

pub fn receive_all(sockfd: &SocketFd) -> Result<Vec<u8>, anyhow::Error> {
    let mut buffer = Vec::new();
    let mut temp_buf = [0u8; 4096];
    loop {
        let bytes_received = sys_libc::recv(sockfd, &mut temp_buf)?;
        if bytes_received.is_none() {
            continue; // would block, try again
        }
        let bytes_received = bytes_received.unwrap();
        if bytes_received == 0 {
            break; // finished receiving
        }
        buffer.extend_from_slice(&temp_buf[..bytes_received]);
    }
    crate::cyan!("Total bytes received: {}", buffer.len());
    Ok(buffer)
}
