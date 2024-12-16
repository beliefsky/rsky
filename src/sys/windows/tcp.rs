use std::{io, net::SocketAddr};

use windows_sys::Win32::Networking::WinSock;

use crate::sys::windows::{Socket, SocketDomain, SocketAddrNative};

pub struct Tcp {
    sock: Socket
}

impl Tcp {

    #[inline]
    pub(crate) fn new<T>(domain: &T) -> io::Result<Self> where T: SocketDomain {
        Socket::new(domain, WinSock::SOCK_STREAM).map(| sock| Self{sock})
    }

    #[inline]
    pub fn  bind<T>(&self, addr: &T) -> io::Result<()> where T: SocketAddrNative{
        self.sock.bind(addr)
    }

    #[inline]
    pub fn listen(&self, backlog: u32) -> io::Result<()> {
        self.sock.listen(backlog)
    }

    pub async fn accept(&self) -> io::Result<(Self, SocketAddr)> {
        Err(io::Error::last_os_error())
    }
}