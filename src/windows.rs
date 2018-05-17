use std::net::{TcpListener, UdpSocket};

pub use std::os::windows::io::{RawHandle, RawSocket};

pub enum FdType {
    Socket(RawSocket),
    Handle(RawHandle),
}

pub fn make_tcp_listener(fd: FdType) -> io::Result<TcpListener> {
    unreachable!()
}

pub fn make_udp_socket(fd: FdType) -> io::Result<UdpSocket> {
    unreachable!()
}

pub fn get_fds() -> Vec<FdType> {
    vec![]
}
