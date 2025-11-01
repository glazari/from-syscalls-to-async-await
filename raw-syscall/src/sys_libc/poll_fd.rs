use super::SocketFd;
use super::libc::pollfd;
use core::fmt::{Debug, Display};
use core::marker::PhantomData;

/// Ties the lifetime of the pollfd to the lifetime of the SocketFd
/// so that we don't have dangling fds in the pollfd array,
/// while maintaining the same memory layout as pollfd
pub struct PollFd<'a>(pollfd, PhantomData<&'a SocketFd>);

impl<'a> PollFd<'a> {
    pub fn new(sockfd: &'a SocketFd, events: i16) -> Self {
        PollFd(
            pollfd {
                fd: sockfd.0,
                events,
                revents: 0,
            },
            PhantomData,
        )
    }

    // SAFETY: The caller must ensure that the output fd will not outlive the
    // underlining SocketFd
    pub unsafe fn fd(&self) -> i32 {
        self.0.fd
    }

    pub fn set_events(&mut self, events: i16) {
        self.0.events = events;
    }

    pub fn revents(&self) -> i16 {
        self.0.revents
    }

    // not sure if this is needed
    pub fn reset_revents(&mut self) {
        self.0.revents = 0;
    }
}

impl Display for PollFd<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "PollFd(fd: {}, events: {:x}, revents: {:x})",
            self.0.fd, self.0.events, self.0.revents
        )
    }
}

impl Debug for PollFd<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "PollFd(fd: {}, events: {:x}, revents: {:x})",
            self.0.fd, self.0.events, self.0.revents
        )
    }
}
