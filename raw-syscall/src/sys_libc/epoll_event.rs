use crate::sys_libc::libc::epoll_data_t;

use super::{SocketFd, libc::epoll_event};
use std::marker::PhantomData;

/// Ties the lifetime of the epoll_event to the lifetime of the SocketFd
/// while maintaining the same memory layout as epoll_event
#[repr(transparent)]
pub struct EpollEvent<'a>(pub(crate) epoll_event, PhantomData<&'a SocketFd>);

impl<'a> EpollEvent<'a> {
    pub fn new(sockfd: &'a SocketFd, events: u32) -> Self {
        EpollEvent(
            epoll_event {
                events,
                data: epoll_data_t {
                    u64: sockfd.0 as u64,
                },
            },
            PhantomData,
        )
    }

    // SAFETY: The caller must ensure that the output fd will not outlive the
    // underlining SocketFd
    pub unsafe fn fd(&self) -> i32 {
        unsafe { self.0.data.u64 as i32 }
    }

    pub fn set_events(&mut self, events: u32) {
        self.0.events = events;
    }

    pub fn events(&self) -> u32 {
        self.0.events
    }
}
