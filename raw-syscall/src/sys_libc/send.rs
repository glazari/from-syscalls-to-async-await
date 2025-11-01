use super::{SocketFd, libc};
use crate::cyan;

pub fn send(sockfd: &SocketFd, buf: &[u8]) -> Result<Option<usize>, anyhow::Error> {
    let bytes_sent = unsafe { libc::send(sockfd.0, buf.as_ptr(), buf.len(), 0) };
    if bytes_sent == -1 {
        let errno = std::io::Error::last_os_error();
        if let Some(os_error) = errno.raw_os_error() {
            if os_error == libc::EWOULDBLOCK || os_error == libc::EAGAIN {
                cyan!("Would block, cannot send data on {}, ({})", sockfd, errno);
                return Ok(None); // Would block, cannot send data
            }
        }
        return Err(anyhow::anyhow!("Failed to send data: {}", errno));
    }
    cyan!("Sent {} bytes", bytes_sent);
    Ok(Some(bytes_sent as usize))
}
