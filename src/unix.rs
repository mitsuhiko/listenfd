use std::env;
use std::io;
use std::mem;
use std::net::{TcpListener, UdpSocket};
use std::os::unix::io::{FromRawFd, RawFd};
use std::os::unix::net::UnixListener;

use libc;

pub type FdType = RawFd;

fn is_sock(fd: FdType) -> bool {
    unsafe {
        let mut stat: libc::stat = mem::zeroed();
        libc::fstat(fd as libc::c_int, &mut stat);
        (stat.st_mode & libc::S_IFMT) == libc::S_IFSOCK
    }
}

fn validate_socket(
    fd: FdType,
    sock_fam: libc::c_int,
    sock_type: libc::c_int,
    hint: &str,
) -> io::Result<FdType> {
    if !is_sock(fd) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("fd {} is not a socket", fd),
        ));
    }

    let is_valid = unsafe {
        let mut ty: libc::c_int = mem::zeroed();
        let mut ty_len = mem::size_of_val(&ty) as libc::c_uint;
        let mut sockaddr: libc::sockaddr = mem::zeroed();
        let mut sockaddr_len = mem::size_of_val(&sockaddr) as libc::c_uint;
        libc::getsockname(fd, &mut sockaddr, &mut sockaddr_len) == 0
            && libc::getsockopt(
                fd,
                libc::SOL_SOCKET,
                libc::SO_TYPE,
                mem::transmute(&mut ty),
                &mut ty_len,
            ) == 0 && ty == sock_type
            && (sockaddr.sa_family == sock_fam as u8
                || (sockaddr.sa_family == libc::AF_INET6 as u8 && sock_fam == libc::AF_INET))
    };

    if !is_valid {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("fd {} is not a valid {}", fd, hint),
        ));
    }

    Ok(fd)
}

pub fn make_tcp_listener(fd: FdType) -> io::Result<TcpListener> {
    validate_socket(fd, libc::AF_INET, libc::SOCK_STREAM, "tcp socket")
        .map(|fd| unsafe { FromRawFd::from_raw_fd(fd) })
}

pub fn make_unix_listener(fd: FdType) -> io::Result<UnixListener> {
    validate_socket(fd, libc::AF_UNIX, libc::SOCK_STREAM, "unix socket")
        .map(|fd| unsafe { FromRawFd::from_raw_fd(fd) })
}

pub fn make_udp_socket(fd: FdType) -> io::Result<UdpSocket> {
    validate_socket(fd, libc::AF_INET, libc::SOCK_DGRAM, "udp socket")
        .map(|fd| unsafe { FromRawFd::from_raw_fd(fd) })
}

pub fn get_fds() -> Vec<FdType> {
    // catflap
    if let Some(fd) = env::var("LISTEN_FD").ok().and_then(|fd| fd.parse().ok()) {
        return vec![fd];
    }

    // systemd
    if env::var("LISTEN_PID").ok().and_then(|x| x.parse().ok()) == Some(unsafe { libc::getpid() }) {
        let count: Option<usize> = env::var("LISTEN_FDS").ok().and_then(|x| x.parse().ok());
        env::remove_var("LISTEN_PID");
        env::remove_var("LISTEN_FDS");
        if let Some(count) = count {
            return (0..count).map(|offset| 3 + offset as FdType).collect();
        }
    }

    vec![]
}
