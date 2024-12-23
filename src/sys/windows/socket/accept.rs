use std::{future::Future, io, net::SocketAddr, sync::OnceLock};

use super::{socket::get_wsa_fn, Socket};

use windows_sys::Win32::Networking::WinSock;

const ACCEPT_ADDR_BUFFER_SIZE: usize = std::mem::size_of::<WinSock::SOCKADDR_STORAGE>() + 16;
const ACCEPT_BUFFER_SIZE: usize = ACCEPT_ADDR_BUFFER_SIZE * 2;

pub struct Accept {
    accept_socket: Socket,
    buffer: [u8; ACCEPT_BUFFER_SIZE],
}

impl Accept {
    pub(crate) fn new(socket: &Socket) -> io::Result<Accept> {
        static ACCEPT_EX: OnceLock<io::Result<WinSock::LPFN_ACCEPTEX>> = OnceLock::new();
        let accept_ref = ACCEPT_EX.get_or_init(|| get_wsa_fn(socket.fd, WinSock::WSAID_ACCEPTEX));

        let accept_fn = match *accept_ref {
            Ok(ref op) => match *op {
                Some(f) => Ok(f),
                None => Err(io::Error::new(
                    io::ErrorKind::Unsupported,
                    "cannot retrieve AcceptEx",
                )),
            },
            Err(ref e) => Err(io::Error::new(e.kind(), e.to_string())),
        }?;

        let accept_socket = Socket::new(socket.domain, socket.socket_type)?;

        let accept = Self {
            accept_socket: accept_socket,
            buffer: unsafe { std::mem::zeroed() },
        };

        // let mut received = 0;
        // unsafe {
        //     accept_fn(
        //         socket.fd,
        //         accept.accept_socket.fd,
        //         accept.buffer.as_mut_ptr(),
        //         0,
        //         ACCEPT_ADDR_BUFFER_SIZE,
        //         ACCEPT_ADDR_BUFFER_SIZE,
        //         &mut received,
        //     )
        // }

        Ok(accept)
    }

    fn test222() -> io::Result<()> {
        Ok(())
    }
}

impl Future for Accept {
    type Output = io::Result<(Socket, SocketAddr)>;
    fn poll(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        std::task::Poll::Pending
    }
}
