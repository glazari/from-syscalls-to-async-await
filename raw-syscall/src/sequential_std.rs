use std::{
    io::{Read, Write},
    net::TcpStream,
};

use crate::green;

pub fn sequential_calls() -> Result<(), anyhow::Error> {
    let start = std::time::Instant::now();
    green!("--- sequential_std ---");
    let request = "GET / HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n";
    for i in 0..3 {
        let resp = make_request("127.0.0.1", 3000, request)?;
        println!("Response:\n{}", String::from_utf8_lossy(&resp));
    }
    println!("Sequential duration: {:?}", start.elapsed());
    Ok(())
}

pub fn make_request(addr: &str, port: u16, request: &str) -> Result<Vec<u8>, anyhow::Error> {
    let mut stream = TcpStream::connect((addr, port))?;
    stream.write_all(request.as_bytes())?;
    let mut response = Vec::new();
    stream.read_to_end(&mut response)?;
    Ok(response)
}
