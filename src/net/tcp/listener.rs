use std::io;
use std::net::SocketAddr;

use crate::net::tcp::TcpStream;

#[cfg(windows)]
use crate::sys::windows::Tcp;

pub struct TcpListener {
    listener: Tcp,
}

impl TcpListener {
    pub fn bind(addr: &SocketAddr) -> io::Result<Self> {
        let tcp = Tcp::new(addr)?;
        tcp.bind(addr)?;
        tcp.listen(1024)?;

        Ok(Self { listener: tcp })
    }

    pub async fn accept(&self) -> io::Result<(TcpStream, SocketAddr)> {
        self.listener
            .accept()
            .await
            .map(|(tcp, addr)| (TcpStream::new(tcp), addr))
    }
}
