use mio::Events;
use mio::Poll;
use mio::event::Source;
use mio::net::TcpStream;
use std::io::{Read, Write};

use crate::green;

pub fn non_blocking_calls() -> Result<(), anyhow::Error> {
    let start = std::time::Instant::now();
    green!("--- non_blocking_mio ---");

    let mut poll = Poll::new()?;
    let mut events = Events::with_capacity(10);

    let mut streams = vec![];
    for i in 0..3 {
        let stream = create_non_blocking_stream("127.0.0.1:3000")?;
        streams.push(stream);
        poll.registry().register(
            &mut streams[i],
            mio::Token(i),
            mio::Interest::READABLE | mio::Interest::WRITABLE,
        )?;
    }

    let request = "GET / HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n";
    send_all(&mut streams, request, &mut poll, &mut events)?;
    let responses = read_all_non_blocking(&mut streams, &mut poll, &mut events)?;

    println!("Received all responses:");
    for (i, response) in responses.iter().enumerate() {
        println!("Response {}:\n{}", i, String::from_utf8_lossy(response));
    }

    println!("Non-blocking mio duration: {:?}", start.elapsed());
    Ok(())
}

fn read_all_non_blocking(
    streams: &mut [TcpStream],
    poll: &mut Poll,
    events: &mut Events,
) -> Result<Vec<Vec<u8>>, anyhow::Error> {
    let mut responses = vec![Vec::new(); streams.len()];
    let mut done = vec![false; streams.len()];
    let timeout = std::time::Duration::from_secs(5);

    // try to read immediately
    for (i, stream) in streams.iter_mut().enumerate() {
        read_until_would_block(stream, &mut responses[i])?;
    }

    while !done.iter().all(|&d| d) {
        poll.poll(events, Some(timeout))?;
        if events.is_empty() {
            return Err(anyhow::anyhow!("Timeout waiting for responses"));
        }
        for event in events.iter() {
            let i = event.token().0;
            if done[i] {
                continue;
            }

            if event.is_readable() {
                let closed = read_until_would_block(&mut streams[i], &mut responses[i]).unwrap();
                if closed {
                    done[i] = true;
                    poll.registry().deregister(&mut streams[i])?;
                }
            } else if event.is_read_closed() {
                done[i] = true;
                let _ = read_until_would_block(&mut streams[i], &mut responses[i]);
                poll.registry().deregister(&mut streams[i])?;
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

fn send_all(
    streams: &mut [TcpStream],
    request: &str,
    poll: &mut Poll,
    events: &mut Events,
) -> Result<(), anyhow::Error> {
    let mut sent = vec![false; streams.len()];
    let timeout = std::time::Duration::from_secs(5);

    // try to send immediately
    for (i, stream) in streams.iter_mut().enumerate() {
        match stream.write(request.as_bytes()) {
            Ok(n) if n == request.len() => {
                sent[i] = true;
            }
            Ok(n) => {
                // partially sent, will try later
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                // not sent yet, will try later
            }
            Err(e) => return Err(anyhow::anyhow!("Error sending on stream {}: {}", i, e)),
        }
    }

    while !sent.iter().all(|&s| s) {
        poll.poll(events, Some(timeout))?;
        for event in events.iter() {
            let i = event.token().0;
            if !sent[i] && event.is_writable() {
                match streams[i].write(request.as_bytes()) {
                    Ok(n) if n == request.len() => {
                        sent[i] = true;
                    }
                    Ok(n) => {
                        // partially sent, will try later
                    }
                    Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                        // not sent yet, will try later
                    }
                    Err(e) => return Err(anyhow::anyhow!("Error sending on stream {}: {}", i, e)),
                }
            }
        }
    }

    Ok(())
}

pub fn create_non_blocking_stream(addr: &str) -> Result<TcpStream, anyhow::Error> {
    let addr = addr.parse().unwrap();
    let stream = TcpStream::connect(addr)?;
    stream.set_nodelay(true)?; // disable Nagle's algorithm
    Ok(stream)
}
