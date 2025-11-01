//! wrapper functions that interact with `socket(2)`
use super::{SocketFd, libc};
use crate::cyan;
use libc::AF_INET;

pub const SOCK_STREAM: i32 = 1;
pub const NON_BLOCKING: i32 = 0o4000;
pub const IPPROTO_TCP: i32 = 6;

pub fn create_tcp_socket() -> Result<SocketFd, anyhow::Error> {
    let sockfd = unsafe { libc::socket(AF_INET, SOCK_STREAM, IPPROTO_TCP) };
    if sockfd == -1 {
        let errno = std::io::Error::last_os_error();
        return Err(anyhow::anyhow!("Failed to create socket: {}", errno));
    }
    let fd = SocketFd(sockfd);
    cyan!("Created socket: {}", fd);
    Ok(fd)
}

pub fn create_non_blocking_tcp_socket() -> Result<SocketFd, anyhow::Error> {
    let sockfd = unsafe { libc::socket(AF_INET, SOCK_STREAM | NON_BLOCKING, IPPROTO_TCP) };
    if sockfd == -1 {
        let errno = std::io::Error::last_os_error();
        return Err(anyhow::anyhow!(
            "Failed to create non-blocking socket: {}",
            errno
        ));
    }
    let fd = SocketFd(sockfd);
    crate::cyan!("Created non-blocking socket: {}", fd);
    Ok(fd)
}
