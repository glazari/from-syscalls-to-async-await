//! External C function and struct definitions for socket programming.
pub const EINPROGRESS: i32 = 115;
pub const EWOULDBLOCK: i32 = 11;
pub const EAGAIN: i32 = 11;

pub const SO_ERROR: i32 = 4;
pub const SOL_SOCKET: i32 = 1;

pub const AF_INET: i32 = 2; // IPv4 family
pub const AF_INET6: i32 = 10; // IPv6 family

// Poll events
pub const POLLIN: i16 = 0x001;
pub const POLLPRI: i16 = 0x002;
pub const POLLOUT: i16 = 0x004;
pub const POLLERR: i16 = 0x008;
pub const POLLHUP: i16 = 0x010;
pub const POLLNVAL: i16 = 0x020;
pub const POLLRDNORM: i16 = 0x040;
pub const POLLRDBAND: i16 = 0x080;
pub const POLLWRNORM: i16 = 0x100;
pub const POLLWRBAND: i16 = 0x200;
pub const POLLMSG: i16 = 0x400;
pub const POLLRDHUP: i16 = 0x2000;

// Socket address structure
#[repr(C)]
pub struct sockaddr_in {
    pub sin_family: u16,
    pub sin_port: u16,
    pub sin_addr: u32,
    pub sin_zero: [u8; 8],
}

/// File descriptor set for select
pub type fd_set = [u64; 16];

#[repr(C)]
pub struct sockaddr {
    pub sa_family: u16,
    pub sa_data: [u8; 14],
}

#[repr(C)]
pub struct pollfd {
    pub fd: i32,
    pub events: i16,
    pub revents: i16,
}

pub type nfds_t = u64;

// --------------------------------------------------
// epoll
// --------------------------------------------------

// op values for epoll_ctl
pub const EPOLL_CTL_ADD: i32 = 1;
pub const EPOLL_CTL_DEL: i32 = 2;
pub const EPOLL_CTL_MOD: i32 = 3;

// events for epoll_event.events (bit mask, to be ORed together)
pub const EPOLLIN: u32 = 0x001;
pub const EPOLLPRI: u32 = 0x002;
pub const EPOLLOUT: u32 = 0x004;
pub const EPOLLERR: u32 = 0x008; // always checked, no need to set
pub const EPOLLHUP: u32 = 0x010; // always checked, no need to set
pub const EPOLLNVAL: u32 = 0x020; // always checked, no need to set
pub const EPOLLRDNORM: u32 = 0x040;
pub const EPOLLRDBAND: u32 = 0x080;
pub const EPOLLWRNORM: u32 = 0x100;
pub const EPOLLWRBAND: u32 = 0x200;
pub const EPOLLMSG: u32 = 0x400;
pub const EPOLLRDHUP: u32 = 0x2000;
pub const EPOLLONESHOT: u32 = 1 << 30;
pub const EPOLLET: u32 = 1 << 31; // Edge Triggered behavior

// epoll_create1 flags
pub const EPOLL_CLOEXEC: i32 = 0o2000000;
pub const EPOLL_NONBLOCK: i32 = 0o4000;

#[repr(C, packed)]
pub struct epoll_event {
    pub events: u32,
    pub data: epoll_data_t,
}

#[repr(C)]
pub union epoll_data_t {
    pub ptr: *mut core::ffi::c_void,
    pub fd: i32,
    pub u32: u32,
    pub u64: u64,
}

// --------------------------------------------------
// External C function declarations
// --------------------------------------------------

unsafe extern "C" {
    // syscalls-ish functions
    pub fn socket(domain: i32, type_: i32, protocol: i32) -> i32;
    pub fn connect(sockfd: i32, addr: *const sockaddr, addrlen: u32) -> i32;
    pub fn send(sockfd: i32, buf: *const u8, len: usize, flags: i32) -> isize;
    pub fn recv(sockfd: i32, buf: *mut u8, len: usize, flags: i32) -> isize;
    pub fn close(fd: i32) -> i32;
    pub fn getsockopt(
        sockfd: i32,
        level: i32,
        optname: i32,
        optval: *mut u8,
        optlen: *mut u32,
    ) -> i32;
    pub fn select(
        nfds: i32,
        readfds: *mut fd_set,
        writefds: *mut fd_set,
        exceptfds: *mut fd_set,
        timeout: *mut u8,
    ) -> i32;
    pub fn poll(fds: *mut pollfd, nfds: nfds_t, timeout: i32) -> i32;
    pub fn epoll_create(size: i32) -> i32;
    pub fn epoll_create1(flags: i32) -> i32;
    pub fn epoll_ctl(epfd: i32, op: i32, fd: i32, event: *mut epoll_event) -> i32;
    pub fn epoll_wait(epfd: i32, events: *mut epoll_event, maxevents: i32, timeout: i32) -> i32;

    // helpers that are not syscalls (could be pure Rust)
    pub fn inet_addr(cp: *const i8) -> u32;
    pub fn htons(hostshort: u16) -> u16;
}
