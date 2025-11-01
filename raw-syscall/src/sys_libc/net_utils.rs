//! wrappers around libc networking functions
//! these don't call any syscalls directly
use super::libc::{self, AF_INET, sockaddr_in};
use std::ffi::CString;

pub fn create_ipv4_sockaddr(addr: &str, port: u16) -> Result<sockaddr_in, anyhow::Error> {
    let addr = parse_network_address(addr)?;
    Ok(sockaddr_in {
        sin_family: AF_INET as u16,
        sin_port: unsafe { libc::htons(port) },
        sin_addr: addr,
        sin_zero: [0; 8],
    })
}

pub fn parse_network_address(addr: &str) -> Result<u32, anyhow::Error> {
    let c_addr = CString::new(addr).unwrap();
    let inet_addr = unsafe { libc::inet_addr(c_addr.as_ptr()) };
    if inet_addr == u32::MAX {
        return Err(anyhow::anyhow!("Invalid IP address {}", addr));
    }
    Ok(inet_addr)
}
