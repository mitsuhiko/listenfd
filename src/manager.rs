use std::io;
use std::net::{TcpListener, UdpSocket};

#[cfg(not(windows))]
use std::os::unix::net::UnixListener;

#[cfg(not(windows))]
use unix as imp;

#[cfg(windows)]
use windows as imp;

/// A helper object that gives access to raw file descriptors.
pub struct ListenFdManager {
    fds: Vec<Option<imp::FdType>>,
}

impl ListenFdManager {
    /// Creates the fd manager from the environment.
    pub fn from_env() -> ListenFdManager {
        ListenFdManager {
            fds: imp::get_fds().into_iter().map(Some).collect(),
        }
    }

    /// Creates an empty fd manager.
    ///
    /// This is helpful when the ability to work with external file
    /// descriptors should be disabled in certain code paths.  This
    /// way the functions on the manager will just never return
    /// sockets.
    pub fn empty() -> ListenFdManager {
        ListenFdManager { fds: vec![] }
    }

    /// Returns the number of fds in the manager.
    ///
    /// Note that even if fds are taken out of the manager this count
    /// does not change.
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
            Some(mut bucket) => bucket,
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
    /// the fd at that position is not a tcp socket then an error is
    /// returned and the fd is left at its place.
    pub fn take_tcp_listener(&mut self, idx: usize) -> io::Result<Option<TcpListener>> {
        self.with_fd(idx, imp::make_tcp_listener)
    }

    /// Takes the UNIX listener at an index.
    ///
    /// If the given index has been used before `Ok(None)` is returned,
    /// otherwise the fd at that index is returned as `UnixListener`.  If
    /// the fd at that position is not a tcp socket then an error is
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
    /// the fd at that position is not a tcp socket then an error is
    /// returned and the fd is left at its place.
    pub fn take_udp_socket(&mut self, idx: usize) -> io::Result<Option<UdpSocket>> {
        self.with_fd(idx, imp::make_udp_socket)
    }

    /// Takes the `RawFd` on unix platforms.
    #[cfg(not(windows))]
    pub fn take_raw_fd(&mut self, idx: usize) -> io::Result<Option<imp::FdType>> {
        self.with_fd(idx, |fd| Ok(fd))
    }

    /// Takes the `RawSocket` on windows platforms.
    ///
    /// This will error if the fd at this position is not a socket (but a handle).
    #[cfg(windows)]
    pub fn take_raw_socket(&mut self, idx: usize) -> io::Result<Option<imp::RawSocket>> {
        Ok(None)
    }

    /// Takes the `RawHandle` on windows platforms.
    ///
    /// This will error if the fd at this position is not a handle.
    #[cfg(windows)]
    pub fn take_raw_handle(&mut self, idx: usize) -> io::Result<Option<imp::RawHandle>> {
        Ok(None)
    }
}
