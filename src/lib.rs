//! listenfd is a crate that provides support for working with externally
//! managed and passed file descriptors.  This lets you work with systems
//! that support socket activation or similar.
//!
//! Currently this supports `systemd` and `catflap` on unix only.  `catflap`
//! is very convenient in combination with cargo-watch for development
//! purposes whereas `systemd` is useful for production deployments on linux.
//!
//! ## Example
//!
//! This example shows how to use this crate with an `actix-web` server:
//!
//! ```rust
//! # use std::io;
//! # struct Server;
//! # impl Server {
//! #   fn listener<X>(self, _: X) -> Self { self }
//! #   fn bind<X>(self, _: X) -> io::Result<Self> { Ok(self) }
//! # }
//! # fn make_a_server() -> Server { Server };
//! # fn test() -> io::Result<()> {
//! use listenfd::ListenFdManager;
//!
//! let mut manager = ListenFdManager::from_env();
//! let mut server = make_a_server();
//!
//! // if we are given a tcp listener on listen fd 0, we use that one
//! server = if let Some(listener) = manager.take_tcp_listener(0)? {
//!     server.listener(listener)
//! // otherwise fall back to local listening
//! } else {
//!     server.bind("127.0.0.1:3000")?
//! };
//! # Ok(()) }
//! ```
//!
//! You can then use this with cargo watch and catflap:
//!
//! ```plain
//! $ cargo install catflap cargo-watch
//! catflap -p 3000 -- cargo watch -x run
//! ```
//!
//! Now catflap will open the socket and keep it open.  cargo watch will
//! recompile the code on demand and the server will pick up the socket
//! that catflap opened.  No more connection resets.
#[cfg(not(windows))]
extern crate libc;

#[cfg(not(windows))]
mod unix;
#[cfg(windows)]
mod windows;

mod manager;

pub use manager::*;
