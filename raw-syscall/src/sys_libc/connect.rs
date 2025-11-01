use super::{SocketFd, libc};
use crate::cyan;
use libc::{sockaddr, sockaddr_in};
use std::mem;

pub fn connect(sockfd: &SocketFd, addr: &sockaddr_in) -> Result<(), anyhow::Error> {
    let result = unsafe {
        let addr = addr as *const sockaddr_in as *const sockaddr;
        let len = mem::size_of::<sockaddr_in>() as u32;
        libc::connect(sockfd.0, addr, len)
    };
    if result == -1 {
        let errno = std::io::Error::last_os_error();
        if errno.raw_os_error() == Some(libc::EINPROGRESS) {
            cyan!("Non-blocking connect in progress for {}", sockfd);
            return Ok(());
        } else {
            return Err(anyhow::anyhow!("Failed to connect {}: {}", sockfd, errno));
        }
    }
    cyan!("Connect {} to {}", sockfd, addr.sin_addr);
    Ok(())
}
