use super::SocketFd;
use super::libc::fd_set;
use std::fmt::Display;
use std::usize;

/// Fdset is a kind of bloom filter for file descriptors
/// If a bit is not set, the fd is definitely not in the set
/// If a bit is set, it might have been set by that fd or by another fd that hashes to the same bit
/// If too many fds are open there can be an out of bounds access??? (realy???)
pub struct FdSet {
    fds: [u64; 16], // 1024 bits / 64 = 16 u64s
}

impl FdSet {
    pub fn new() -> Self {
        Self { fds: [0; 16] }
    }

    pub fn clear(&mut self) {
        self.fds.fill(0);
    }

    pub fn set(&mut self, fd: &SocketFd) {
        let (idx, bit) = Self::idx_bit(fd);
        if idx < self.fds.len() {
            self.fds[idx] |= bit;
        }
    }

    pub fn is_set(&self, fd: &SocketFd) -> bool {
        let (idx, bit) = Self::idx_bit(fd);
        if idx < self.fds.len() {
            (self.fds[idx] & bit) != 0
        } else {
            false
        }
    }

    fn idx_bit(fd: &SocketFd) -> (usize, u64) {
        let fd = fd.0 as usize;
        let idx = fd / 64;
        let bit = 1u64 << (fd % 64);
        (idx, bit)
    }

    pub(crate) fn as_mut_ptr(&mut self) -> *mut fd_set {
        self.fds.as_mut_ptr() as *mut fd_set
    }
}

impl Display for FdSet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut bits = Vec::new();
        for (i, &chunk) in self.fds.iter().enumerate() {
            for j in 0..64 {
                if (chunk & (1u64 << j)) != 0 {
                    bits.push(i * 64 + j);
                }
            }
        }
        write!(f, "FdSet({:?})", bits)
    }
}
