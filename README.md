# listenfd

[![Build Status](https://github.com/mitsuhiko/listenfd/workflows/Tests/badge.svg?branch=master)](https://github.com/mitsuhiko/listenfd/actions?query=workflow%3ATests)
[![Crates.io](https://img.shields.io/crates/d/listenfd.svg)](https://crates.io/crates/listenfd)
[![License](https://img.shields.io/github/license/mitsuhiko/listenfd)](https://github.com/mitsuhiko/listenfd/blob/master/LICENSE)
[![rustc 1.42.0](https://img.shields.io/badge/rust-1.42%2B-orange.svg)](https://img.shields.io/badge/rust-1.42%2B-orange.svg)
[![Documentation](https://docs.rs/listenfd/badge.svg)](https://docs.rs/listenfd)

listenfd is a crate that provides support for working with externally managed
and passed file descriptors. This lets you work with systems that support
socket activation or similar.

Currently this supports systemd (including systemd-socket-activate) on Unix and
[systemfd](https://github.com/mitsuhiko/systemfd) on Unix and Windows.
systemfd is very convenient in combination with cargo-watch for development
purposes whereas systemd is useful for production deployments on linux.

## Example

```rust
use listenfd::ListenFd;

let mut listenfd = ListenFd::from_env();
let mut server = make_a_server();

// if we are given a tcp listener on listen fd 0, we use that one
server = if let Some(listener) = listenfd.take_tcp_listener(0)? {
    server.listen(listener)
// otherwise fall back to local listening
} else {
    server.bind("127.0.0.1:3000")?
};
```

You can then use this with cargo watch and systemfd:

```
$ cargo install systemfd cargo-watch
systemfd --no-pid -s http::3000 -- cargo watch -x run
```

Now systemfd will open the socket and keep it open. cargo watch will recompile
the code on demand and the server will pick up the socket that systemfd opened.
No more connection resets.

## License and Links

- [Documentation](https://docs.rs/listenfd/)
- [Issue Tracker](https://github.com/mitsuhiko/listenfd/issues)
- [Examples](https://github.com/mitsuhiko/listenfd/tree/main/examples)
- License: [Apache-2.0](https://github.com/mitsuhiko/listenfd/blob/main/LICENSE)
