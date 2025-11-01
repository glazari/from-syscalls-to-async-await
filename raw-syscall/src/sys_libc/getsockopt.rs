use super::SocketFd;
use super::libc;
use std::mem;

pub fn get_socket_error(sockfd: &SocketFd) -> Result<i32, anyhow::Error> {
    let mut error: i32 = 0;
    let mut len = mem::size_of::<i32>() as u32;
    let mut optval = error.to_ne_bytes();
    let _ = getsockopt(sockfd, libc::SOL_SOCKET, libc::SO_ERROR, &mut optval)?;
    Ok(error)
}

pub fn getsockopt(
    sockfd: &SocketFd,
    level: i32,
    optname: i32,
    optval: &mut [u8],
) -> Result<usize, anyhow::Error> {
    let mut optlen = optval.len() as u32;
    let ret =
        unsafe { libc::getsockopt(sockfd.0, level, optname, optval.as_mut_ptr(), &mut optlen) };
    if ret == -1 {
        let errno = std::io::Error::last_os_error();
        return Err(anyhow::anyhow!("Failed to get socket option: {}", errno));
    }
    Ok(optlen as usize)
}
