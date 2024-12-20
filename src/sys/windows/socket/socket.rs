use std::{io, net::SocketAddr, sync::Once};

use super::{accept::Accept, SocketAddrNative, SocketDomain};

use windows_sys::{core::GUID, Win32::Networking::WinSock};

const ACCEPT_ADDR_BUFFER_SIZE: usize = std::mem::size_of::<WinSock::SOCKADDR_STORAGE>() + 16;
const ACCEPT_BUFFER_SIZE: usize = ACCEPT_ADDR_BUFFER_SIZE * 2;

pub struct Socket {
    pub(crate) fd: WinSock::SOCKET,
    pub(crate) domain: i32,
    pub(crate) socket_type: i32,
}

impl Socket {
    pub(crate) fn new<T: SocketDomain>(domain: T, socket_type: i32) -> io::Result<Self>
    where
        T: SocketDomain,
    {
        {
            static INIT: Once = Once::new();
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
            Ok(Self {
                fd: fd,
                domain: af,
                socket_type: socket_type,
            })
        } else {
            unsafe {
                WinSock::closesocket(fd);
            }
            Err(io::Error::last_os_error())
        }
    }

    pub fn bind<T>(&self, addr: T) -> io::Result<()>
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
    pub async fn accept(&self) -> io::Result<(Self, SocketAddr)> {
        Accept::new(self)?.await
    }
}

impl Drop for Socket {
    fn drop(&mut self) {
        unsafe {
            WinSock::closesocket(self.fd);
        }
    }
}

pub(crate) fn get_wsa_fn<F>(fd: WinSock::SOCKET, guid: GUID) -> io::Result<Option<F>> {
    let mut fptr = None;
    let mut returned = 0;

    if unsafe {
        WinSock::WSAIoctl(
            fd,
            WinSock::SIO_GET_EXTENSION_FUNCTION_POINTER,
            std::ptr::addr_of!(guid).cast(),
            std::mem::size_of_val(&guid) as _,
            std::ptr::addr_of_mut!(fptr).cast(),
            std::mem::size_of::<F>() as _,
            &mut returned,
            std::ptr::null_mut(),
            None,
        )
    } != 0
    {
        Err(std::io::Error::last_os_error())
    } else {
        Ok(fptr)
    }
}
