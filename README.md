# listenfd

<a href="https://travis-ci.com/mitsuhiko/rust-listenfd"><img src="https://travis-ci.com/mitsuhiko/rust-listenfd.svg?branch=master" alt=""></a>
<a href="https://crates.io/crates/listenfd"><img src="https://img.shields.io/crates/v/listenfd.svg" alt=""></a>

listenfd is a crate that provides support for working with externally managed
and passed file descriptors. This lets you work with systems that support
socket activation or similar.

Currently this supports systemd and catflap on unix only.  catflap is very
convenient in combination with cargo-watch for development purposes whereas
systemd is useful for production deployments on linux.

## Example

```rust
extern crate listenfd;
use listenfd::ListenFdManager;

let mut manager = ListenFdManager::from_env();
let mut server = make_a_server();

// if we are given a tcp listener on listen fd 0, we use that one
server = if let Some(listener) = manager.take_tcp_listener(0)? {
    server.listener(listener)
// otherwise fall back to local listening
} else {
    server.bind("127.0.0.1:3000")?
};
```

You can then use this with cargo watch and catflap:

```
$ cargo install catflap cargo-watch
catflap -p 3000 -- cargo watch -x run
```

Now catflap will open the socket and keep it open. cargo watch will recompile
the code on demand and the server will pick up the socket that catflap opened.
No more connection resets.

## License

Symbolic is licensed under the Apache 2 license.
