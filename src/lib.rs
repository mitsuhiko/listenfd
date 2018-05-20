//! listenfd is a crate that provides support for working with externally
//! managed and passed file descriptors.  This lets you work with systems
//! that support socket activation or similar.
//!
//! Currently this supports a slightly modified systemd protocl on unix and
//! a custom protocol on Windows.  If you want to use this for development
//! you can use the [systemfd](https://github.com/mitsuhiko/systemfd)
//! utility which implements both those protocols.
//!
//! The systemd extension is that if the `LISTEN_PID` variable is not set or
//! empty, the check for the pid is removed.  This is useful when binaries
//! are proxied in between like cargo-watch.  For the windows protocol
//! have a look at the systemfd documentation.
//!
//! ## Example
//!
//! This example shows how to use this crate with an `actix-web` server:
//!
//! ```rust
//! # use std::io;
//! # struct Server;
//! # impl Server {
//! #   fn listen<X>(self, _: X) -> Self { self }
//! #   fn bind<X>(self, _: X) -> io::Result<Self> { Ok(self) }
//! # }
//! # fn make_a_server() -> Server { Server };
//! # fn test() -> io::Result<()> {
//! use listenfd::ListenFd;
//!
//! let mut listenfd = ListenFd::from_env();
//! let mut server = make_a_server();
//!
//! // if we are given a tcp listener on listen fd 0, we use that one
//! server = if let Some(listener) = listenfd.take_tcp_listener(0)? {
//!     server.listen(listener)
//! // otherwise fall back to local listening
//! } else {
//!     server.bind("127.0.0.1:3000")?
//! };
//! # Ok(()) }
//! ```
//!
//! You can then use this with cargo watch and systemfd:
//!
//! ```plain
//! $ cargo install systemfd cargo-watch
//! systemfd --no-pid -s 3000 -- cargo watch -x run
//! ```
//!
//! Now systemfd will open the socket and keep it open.  cargo watch will
//! recompile the code on demand and the server will pick up the socket
//! that systemfd opened.  No more connection resets.
//!
//! The `--no-pid` flag is necessary to ensure that the `LISTEN_PID` environment
//! variable is not set or the socket passing will be prevented by the pid check.
#[cfg(not(windows))]
extern crate libc;
#[cfg(windows)]
extern crate uuid;
#[cfg(windows)]
extern crate winapi;

#[cfg(not(windows))]
mod unix;
#[cfg(windows)]
mod windows;

mod manager;
pub use manager::*;
