pub(crate) type SaFamily = u16;
pub(crate) type Socklen = u32;

pub(crate) const AF_UNSPEC: u32 = 0;
pub(crate) const AF_INET: u32 = 2;
pub(crate) const AF_INET6: u32 = 10;

pub(crate) const MSG_OOB: u32 = 1;
pub(crate) const MSG_PEEK: u32 = 2;
pub(crate) const MSG_NOSIGNAL: u32 = 0x4000;

pub(crate) const SHUT_RD: i32 = 0;
pub(crate) const SHUT_WR: i32 = 1;
pub(crate) const SHUT_RDWR: i32 = 2;

#[cfg(target_arch = "x86_64")]
pub(crate) use x86_64::*;

#[cfg(target_arch = "x86_64")]
mod x86_64 {
    pub(crate) const SOCK_STREAM: u32 = 1;
    pub(crate) const SOCK_DGRAM: u32 = 2;

    pub(crate) const IPPROTO_TCP: u32 = 6;
    pub(crate) const IPPROTO_UDP: u32 = 17;

    pub(crate) const O_CLOEXEC: i32 = 0x80000;
}
