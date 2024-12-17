#[cfg(windows)]
use crate::sys::windows::Tcp;

pub struct TcpStream {
    tcp: Tcp,
}

impl TcpStream {
    pub(crate) fn new(tcp: Tcp) -> Self {
        Self { tcp }
    }
}
