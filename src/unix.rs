use std::env;
use std::io;
use std::mem;
use std::net::{TcpListener, UdpSocket};
use std::os::unix::io::{FromRawFd, RawFd};
use std::os::unix::net::{UnixDatagram, UnixListener};

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
            ) == 0
            && ty == sock_type
            && (sockaddr.sa_family as libc::c_int == sock_fam
                || (sockaddr.sa_family as libc::c_int == libc::AF_INET6
                    && sock_fam == libc::AF_INET))
    };

    if !is_valid {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("fd {} is not a valid {}", fd, hint),
        ));
    }

    Ok(fd)
}

fn mark_cloexec(fd: i32) -> io::Result<i32> {
    let rv = unsafe { libc::fcntl(fd, libc::F_SETFD, libc::FD_CLOEXEC) };
    if rv < 0 {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            format!(
                "fd {} cannot be marked as FD_CLOEXEC ({})",
                fd,
                io::Error::last_os_error()
            ),
        ));
    }
    Ok(fd)
}

pub fn make_tcp_listener(fd: FdType) -> io::Result<TcpListener> {
    validate_socket(fd, libc::AF_INET, libc::SOCK_STREAM, "tcp socket")
        .and_then(mark_cloexec)
        .map(|fd| unsafe { FromRawFd::from_raw_fd(fd) })
}

pub fn make_unix_listener(fd: FdType) -> io::Result<UnixListener> {
    validate_socket(fd, libc::AF_UNIX, libc::SOCK_STREAM, "unix stream socket")
        .and_then(mark_cloexec)
        .map(|fd| unsafe { FromRawFd::from_raw_fd(fd) })
}

pub fn make_udp_socket(fd: FdType) -> io::Result<UdpSocket> {
    validate_socket(fd, libc::AF_INET, libc::SOCK_DGRAM, "udp socket")
        .and_then(mark_cloexec)
        .map(|fd| unsafe { FromRawFd::from_raw_fd(fd) })
}

pub fn make_unix_datagram(fd: FdType) -> io::Result<UnixDatagram> {
    validate_socket(fd, libc::AF_UNIX, libc::SOCK_DGRAM, "unix datagram socket")
        .and_then(mark_cloexec)
        .map(|fd| unsafe { FromRawFd::from_raw_fd(fd) })
}

pub fn make_custom<T: FromRawFd>(
    fd: FdType,
    sock_fam: libc::c_int,
    sock_type: libc::c_int,
    hint: &str,
) -> io::Result<T> {
    validate_socket(fd, sock_fam, sock_type, hint)
        .and_then(mark_cloexec)
        .map(|fd| unsafe { FromRawFd::from_raw_fd(fd) })
}

pub fn get_fds() -> Option<Vec<FdType>> {
    // modified systemd protocol
    if let Some(count) = env::var("LISTEN_FDS").ok().and_then(|x| x.parse().ok()) {
        let ok = match env::var("LISTEN_PID").as_ref().map(|x| x.as_str()) {
            Err(env::VarError::NotPresent) | Ok("") => true,
            Ok(val) if val.parse().ok() == Some(unsafe { libc::getpid() }) => true,
            _ => false,
        };

        let first_fd = env::var("LISTEN_FDS_FIRST_FD").ok().and_then(|x| x.parse().ok()).unwrap_or(3);

        env::remove_var("LISTEN_PID");
        env::remove_var("LISTEN_FDS");
        if ok {
            return Some((0..count).map(|offset| first_fd + offset as FdType).collect());
        }
    }

    None
}
