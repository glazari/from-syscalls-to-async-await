//! Socket file descriptor wrapper
//! Closes the socket when droppeduse super::libc;
use super::libc;
use crate::cyan;
use std::fmt::Display;

pub struct SocketFd(pub(crate) i32);

impl PartialEq for SocketFd {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl PartialOrd for SocketFd {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.0.partial_cmp(&other.0)
    }
}

impl Drop for SocketFd {
    fn drop(&mut self) {
        unsafe { libc::close(self.0) };
        cyan!("Socket closed {}", self);
    }
}
impl Display for SocketFd {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "SocketFd({})", self.0)
    }
}
