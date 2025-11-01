use super::libc::pollfd;
use super::{PollFd, SocketFd};

pub fn poll(fds: &mut [PollFd], timeout: i32) -> Result<i32, anyhow::Error> {
    let nfds = fds.len() as super::libc::nfds_t;
    let result = unsafe { super::libc::poll(fds.as_mut_ptr() as *mut pollfd, nfds, timeout) };

    if result == -1 {
        let errno = std::io::Error::last_os_error();
        return Err(anyhow::anyhow!("Poll failed: {}", errno));
    }

    Ok(result)
}
