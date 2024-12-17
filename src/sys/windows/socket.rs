use std::{
    io,
    net::{SocketAddr, SocketAddrV4, SocketAddrV6},
};

use windows_sys::Win32::Networking::WinSock;

pub struct Socket {
    fd: WinSock::SOCKET,
}

impl Socket {
    pub(crate) fn new<T: SocketDomain>(domain: &T, socket_type: i32) -> io::Result<Self>
    where
        T: SocketDomain,
    {
        {
            static INIT: std::sync::Once = std::sync::Once::new();
            INIT.call_once(|| {
                let ret = unsafe {
                    let mut wsa: WinSock::WSADATA = std::mem::zeroed();
                    WinSock::WSAStartup(
                        0x202, // version 2.2,
                        &mut wsa,
                    )
                };
                assert_eq!(ret, 0);
            });
        }

        let af: i32 = domain.doamin();

        let fd = unsafe {
            WinSock::WSASocketW(
                af,
                socket_type,
                0,
                std::ptr::null_mut(),
                0,
                WinSock::WSA_FLAG_OVERLAPPED | WinSock::WSA_FLAG_NO_HANDLE_INHERIT,
            )
        };

        if fd == WinSock::INVALID_SOCKET {
            return Err(io::Error::last_os_error());
        }

        let success = unsafe { WinSock::ioctlsocket(fd, WinSock::FIONBIO, &mut 1) } == 0;

        if success {
            Ok(Self { fd })
        } else {
            unsafe {
                WinSock::closesocket(fd);
            }
            Err(io::Error::last_os_error())
        }
    }

    pub fn bind<T>(&self, addr: &T) -> io::Result<()>
    where
        T: SocketAddrNative,
    {
        let (natvie_addr, addr_len) = addr.native_addr();

        let success = unsafe { WinSock::bind(self.fd, natvie_addr.as_ptr(), addr_len) } == 0;

        if success {
            Ok(())
        } else {
            Err(io::Error::last_os_error())
        }
    }

    pub fn listen(&self, backlog: u32) -> io::Result<()> {
        let backlog = backlog.try_into().unwrap_or(i32::MAX);

        let success = unsafe { WinSock::listen(self.fd, backlog) } == 0;

        if success {
            Ok(())
        } else {
            Err(io::Error::last_os_error())
        }
    }

    // pub fn accept(&self) -> io::Result<(Socket, SocketAddr)> {

    // }
}

impl Drop for Socket {
    fn drop(&mut self) {
        unsafe {
            WinSock::closesocket(self.fd);
        }
    }
}

pub trait SocketDomain {
    fn doamin(&self) -> i32;
}

impl SocketDomain for SocketAddr {
    #[inline]
    fn doamin(&self) -> i32 {
        match self {
            SocketAddr::V4(..) => WinSock::AF_INET as i32,
            SocketAddr::V6(..) => WinSock::AF_INET6 as i32,
        }
    }
}

impl SocketDomain for SocketAddrV4 {
    #[inline]
    fn doamin(&self) -> i32 {
        WinSock::AF_INET as i32
    }
}

impl SocketDomain for SocketAddrV6 {
    #[inline]
    fn doamin(&self) -> i32 {
        WinSock::AF_INET6 as i32
    }
}

impl SocketDomain for u32 {
    #[inline]
    fn doamin(&self) -> i32 {
        *self as i32
    }
}

impl SocketDomain for i32 {
    #[inline]
    fn doamin(&self) -> i32 {
        *self
    }
}

#[repr(C)]
pub union SocketAddrCRepr {
    v4: WinSock::SOCKADDR_IN,
    v6: WinSock::SOCKADDR_IN6,
}

impl SocketAddrCRepr {
    #[inline]
    pub const fn as_ptr(&self) -> *const WinSock::SOCKADDR {
        self as *const _ as *const WinSock::SOCKADDR
    }
}

pub trait SocketAddrNative {
    fn native_addr(&self) -> (SocketAddrCRepr, i32);
}

impl SocketAddrNative for SocketAddr {
    #[inline]
    fn native_addr(&self) -> (SocketAddrCRepr, i32) {
        match self {
            SocketAddr::V4(ref addr) => addr.native_addr(),
            SocketAddr::V6(ref addr) => addr.native_addr(),
        }
    }
}

impl SocketAddrNative for SocketAddrV4 {
    fn native_addr(&self) -> (SocketAddrCRepr, i32) {
        let sin_addr = unsafe {
            let mut s_un = std::mem::zeroed::<WinSock::IN_ADDR_0>();
            s_un.S_addr = u32::from_ne_bytes(self.ip().octets());
            WinSock::IN_ADDR { S_un: s_un }
        };

        let sockaddr_in = WinSock::SOCKADDR_IN {
            sin_family: WinSock::AF_INET as u16, // 1
            sin_port: self.port().to_be(),
            sin_addr,
            sin_zero: [0; 8],
        };

        let sockaddr = SocketAddrCRepr { v4: sockaddr_in };
        (sockaddr, std::mem::size_of::<WinSock::SOCKADDR_IN>() as i32)
    }
}

impl SocketAddrNative for SocketAddrV6 {
    fn native_addr(&self) -> (SocketAddrCRepr, i32) {
        let sin6_addr = unsafe {
            let mut u = std::mem::zeroed::<WinSock::IN6_ADDR_0>();
            u.Byte = self.ip().octets();
            WinSock::IN6_ADDR { u }
        };
        let u = unsafe {
            let mut u = std::mem::zeroed::<WinSock::SOCKADDR_IN6_0>();
            u.sin6_scope_id = self.scope_id();
            u
        };

        let sockaddr_in6 = WinSock::SOCKADDR_IN6 {
            sin6_family: WinSock::AF_INET6 as u16, // 23
            sin6_port: self.port().to_be(),
            sin6_addr,
            sin6_flowinfo: self.flowinfo(),
            Anonymous: u,
        };

        let sockaddr = SocketAddrCRepr { v6: sockaddr_in6 };
        (
            sockaddr,
            std::mem::size_of::<WinSock::SOCKADDR_IN6>() as i32,
        )
    }
}
