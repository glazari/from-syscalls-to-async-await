use super::{FdSet, libc};

pub fn select_write(
    nfds: i32,
    writefds: &mut FdSet,
    timeout: Option<&mut u8>,
) -> Result<i32, anyhow::Error> {
    select(nfds, None, Some(writefds), None, timeout)
}

pub fn select_read(
    nfds: i32,
    readfds: &mut FdSet,
    timeout: Option<&mut u8>,
) -> Result<i32, anyhow::Error> {
    select(nfds, Some(readfds), None, None, timeout)
}

pub fn select(
    nfds: i32,
    readfds: Option<&mut FdSet>,
    writefds: Option<&mut FdSet>,
    exceptfds: Option<&mut FdSet>,
    timeout: Option<&mut u8>,
) -> Result<i32, anyhow::Error> {
    let readfds = get_mut_ptr(readfds);
    let writefds = get_mut_ptr(writefds);
    let exceptfds = get_mut_ptr(exceptfds);
    let timeout = match timeout {
        Some(t) => t as *mut u8,
        None => std::ptr::null_mut(),
    };
    let result = unsafe { libc::select(nfds, readfds, writefds, exceptfds, timeout) };

    if result == -1 {
        let errno = std::io::Error::last_os_error();
        return Err(anyhow::anyhow!("Select failed: {}", errno));
    }

    Ok(result)
}

fn get_mut_ptr(fd_set: Option<&mut FdSet>) -> *mut libc::fd_set {
    match fd_set {
        Some(fd_set) => fd_set.as_mut_ptr(),
        None => std::ptr::null_mut(),
    }
}
