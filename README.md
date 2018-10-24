# listenfd

<a href="https://travis-ci.com/mitsuhiko/rust-listenfd"><img src="https://travis-ci.com/mitsuhiko/rust-listenfd.svg?branch=master" alt=""></a>
<a href="https://crates.io/crates/listenfd"><img src="https://img.shields.io/crates/v/listenfd.svg" alt=""></a>

listenfd is a crate that provides support for working with externally managed
and passed file descriptors. This lets you work with systems that support
socket activation or similar.

Currently this supports systemd on Unix and
[systemfd](https://github.com/mitsuhiko/systemfd) on Unix and Windows.
systemfd is very convenient in combination with cargo-watch for development
purposes whereas systemd is useful for production deployments on linux.

## Example

```rust
extern crate listenfd;
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
systemfd --no-pid -p 3000 -- cargo watch -x run
```

Now systemfd will open the socket and keep it open. cargo watch will recompile
the code on demand and the server will pick up the socket that systemfd opened.
No more connection resets.
