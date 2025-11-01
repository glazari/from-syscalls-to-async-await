use super::{SocketFd, libc};
use crate::cyan;

/// Receive data from a socket in a (possibly) non-blocking manner.
/// If no data is available or would block, returns Ok(None).
/// On success, returns Ok(Some(bytes_received)).
/// On error, returns Err with the error details.
pub fn recv(sockfd: &SocketFd, buf: &mut [u8]) -> Result<Option<usize>, anyhow::Error> {
    let bytes_received = unsafe { libc::recv(sockfd.0, buf.as_mut_ptr(), buf.len(), 0) };
    if bytes_received == -1 {
        let errno = std::io::Error::last_os_error();
        if let Some(os_error) = errno.raw_os_error() {
            if os_error == libc::EWOULDBLOCK || os_error == libc::EAGAIN {
                cyan!("Would block, no data available on {}, ({})", sockfd, errno);
                return Ok(None); // Would block, no data available
            }
        }
        return Err(anyhow::anyhow!("Failed to receive data: {}", errno));
    }
    cyan!("Received {} bytes", bytes_received);
    Ok(Some(bytes_received as usize))
}
