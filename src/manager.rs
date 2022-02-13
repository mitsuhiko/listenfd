use std::io;
use std::net::{TcpListener, UdpSocket};

#[cfg(not(windows))]
use std::os::unix::net::{UnixDatagram, UnixListener};

#[cfg(not(windows))]
use crate::unix as imp;

#[cfg(windows)]
use crate::windows as imp;

/// A helper object that gives access to raw file descriptors.
pub struct ListenFd {
    fds: Vec<Option<imp::FdType>>,
}

impl ListenFd {
    /// Creates the listenfd manager object from the environment.
    pub fn from_env() -> ListenFd {
        match imp::get_fds() {
            Some(fds) => ListenFd {
                fds: fds.into_iter().map(Some).collect(),
            },
            None => ListenFd::empty(),
        }
    }

    /// Creates an empty listenfd object.
    ///
    /// This is helpful when the ability to work with external file
    /// descriptors should be disabled in certain code paths.  This
    /// way the functions on the object will just never return
    /// sockets.
    pub fn empty() -> ListenFd {
        ListenFd { fds: vec![] }
    }

    /// Returns the number of fds in the manager object.
    ///
    /// Note that even if fds are taken out of the manager this count
    /// does not change.
    #[allow(clippy::len_without_is_empty)]
    pub fn len(&self) -> usize {
        self.fds.len()
    }

    fn with_fd<R, F: FnOnce(imp::FdType) -> io::Result<R>>(
        &mut self,
        idx: usize,
        f: F,
    ) -> io::Result<Option<R>> {
        let bucket = match self.fds.get_mut(idx) {
            Some(None) | None => return Ok(None),
            Some(bucket) => bucket,
        };
        f(*bucket.as_ref().unwrap()).map(|rv| {
            bucket.take();
            Some(rv)
        })
    }

    /// Takes the TCP listener at an index.
    ///
    /// If the given index has been used before `Ok(None)` is returned,
    /// otherwise the fd at that index is returned as `TcpListener`.  If
    /// the fd at that position is not a TCP socket then an error is
    /// returned and the fd is left at its place.
    pub fn take_tcp_listener(&mut self, idx: usize) -> io::Result<Option<TcpListener>> {
        self.with_fd(idx, imp::make_tcp_listener)
    }

    /// Takes the UNIX stream listener at an index.
    ///
    /// If the given index has been used before `Ok(None)` is returned,
    /// otherwise the fd at that index is returned as `UnixListener`.  If
    /// the fd at that position is not a UNIX stream socket then an error is
    /// returned and the fd is left at its place.
    ///
    /// This function is only available on unix platforms.
    #[cfg(not(windows))]
    pub fn take_unix_listener(&mut self, idx: usize) -> io::Result<Option<UnixListener>> {
        self.with_fd(idx, imp::make_unix_listener)
    }

    /// Takes the UDP socket at an index.
    ///
    /// If the given index has been used before `Ok(None)` is returned,
    /// otherwise the fd at that index is returned as `UdpSocket`.  If
    /// the fd at that position is not a UDP socket then an error is
    /// returned and the fd is left at its place.
    pub fn take_udp_socket(&mut self, idx: usize) -> io::Result<Option<UdpSocket>> {
        let _idx = idx;
        self.with_fd(idx, imp::make_udp_socket)
    }

    /// Takes the UNIX datagram socket at an index.
    ///
    /// If the given index has been used before `Ok(None)` is returned,
    /// otherwise the fd at that index is returned as `UnixDatagram`.  If
    /// the fd at that position is not a UNIX datagram socket then an error is
    /// returned and the fd is left at its place.
    ///
    /// This function is only available on unix platforms.
    #[cfg(not(windows))]
    pub fn take_unix_datagram(&mut self, idx: usize) -> io::Result<Option<UnixDatagram>> {
        self.with_fd(idx, imp::make_unix_datagram)
    }

    /// Takes a custom socket on unix platforms.
    ///
    /// You have to provide a socket family, socket type, and a hint
    /// for the validation error, e.g.
    /// `libc::AF_UNIX, libc::SOCK_SEQPACKET, "unix seqpacket socket"`.
    ///
    /// The file descriptor will be validated to actually be a socket
    /// with the appropriate options, set as CLOEXEC, and converted to
    /// the given Rust type using `FromRawFd`.
    ///
    /// This function is only available on unix platforms.
    #[cfg(not(windows))]
    pub fn take_custom<T: std::os::unix::prelude::FromRawFd>(
        &mut self,
        idx: usize,
        sock_fam: libc::c_int,
        sock_type: libc::c_int,
        hint: &str,
    ) -> io::Result<Option<T>> {
        self.with_fd(idx, |fd| imp::make_custom(fd, sock_fam, sock_type, hint))
    }

    /// Takes the `RawFd` on unix platforms.
    #[cfg(not(windows))]
    pub fn take_raw_fd(&mut self, idx: usize) -> io::Result<Option<imp::FdType>> {
        let _idx = idx;
        self.with_fd(idx, Ok)
    }

    /// Takes the `RawSocket` on windows platforms.
    ///
    /// This will error if the fd at this position is not a socket (but a handle).
    #[cfg(windows)]
    pub fn take_raw_socket(&mut self, idx: usize) -> io::Result<Option<imp::RawSocket>> {
        let _idx = idx;
        Ok(None)
    }
}
