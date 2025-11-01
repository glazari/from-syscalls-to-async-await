use super::libc::epoll_event;
use crate::sys_libc::{EpollFd, SocketFd, epoll_event::EpollEvent};

pub fn epoll_create1(flags: i32) -> Result<EpollFd, anyhow::Error> {
    let epoll_fd = unsafe { super::libc::epoll_create1(flags) };
    if epoll_fd == -1 {
        let errno = std::io::Error::last_os_error();
        return Err(anyhow::anyhow!("epoll_create1 failed: {}", errno));
    }
    Ok(EpollFd(epoll_fd))
}

pub fn epoll_ctl(
    epoll_fd: &EpollFd,
    op: i32,
    fd: &SocketFd,
    event: &EpollEvent,
) -> Result<(), anyhow::Error> {
    let event = &event.0 as *const epoll_event as *mut epoll_event;
    let result = unsafe { super::libc::epoll_ctl(epoll_fd.0, op, fd.0, event) };
    if result == -1 {
        let errno = std::io::Error::last_os_error();
        return Err(anyhow::anyhow!("epoll_ctl failed: {}", errno));
    }
    Ok(())
}

pub fn epoll_ctl_remove(epoll_fd: &EpollFd, fd: &SocketFd) -> Result<(), anyhow::Error> {
    let event = std::ptr::null_mut();
    let result =
        unsafe { super::libc::epoll_ctl(epoll_fd.0, super::libc::EPOLL_CTL_DEL, fd.0, event) };
    if result == -1 {
        let errno = std::io::Error::last_os_error();
        return Err(anyhow::anyhow!("epoll_ctl remove failed: {}", errno));
    }
    Ok(())
}

pub fn epoll_wait(
    epoll_fd: &EpollFd,
    events: &mut [epoll_event],
    timeout: i32,
) -> Result<i32, anyhow::Error> {
    let maxevents = events.len() as i32;
    let result =
        unsafe { super::libc::epoll_wait(epoll_fd.0, events.as_mut_ptr(), maxevents, timeout) };
    if result == -1 {
        let errno = std::io::Error::last_os_error();
        return Err(anyhow::anyhow!("epoll_wait failed: {}", errno));
    }
    Ok(result)
}
