use std::io;
use std::net::{TcpListener, UdpSocket};

pub use std::os::windows::io::{RawHandle, RawSocket};

#[derive(Copy, Clone, Debug)]
pub enum FdType {
    Socket(RawSocket),
    Handle(RawHandle),
}

pub fn make_tcp_listener(_fd: FdType) -> io::Result<TcpListener> {
    unreachable!()
}

pub fn make_udp_socket(_fd: FdType) -> io::Result<UdpSocket> {
    unreachable!()
}

pub fn get_fds() -> Vec<FdType> {
    vec![]
}
