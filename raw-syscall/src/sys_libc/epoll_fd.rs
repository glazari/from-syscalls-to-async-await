//! Epoll file descriptor wrapper.
//! Closes the epoll instance when dropped.

use super::libc;
use crate::cyan;
use std::fmt::Display;

pub struct EpollFd(pub(crate) i32);

impl PartialEq for EpollFd {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl Drop for EpollFd {
    fn drop(&mut self) {
        unsafe { libc::close(self.0) };
        cyan!("Epoll instance closed {}", self);
    }
}

impl Display for EpollFd {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "EpollFd({})", self.0)
    }
}
