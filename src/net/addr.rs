use std::net::{SocketAddr, SocketAddrV4, SocketAddrV6};



pub trait ToSocketAddrs {

}

impl ToSocketAddrs for SocketAddr {
    
}

impl ToSocketAddrs for SocketAddrV4 {
    
}

impl ToSocketAddrs for SocketAddrV6 {
    
}