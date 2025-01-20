# warp Example

To run this example with systemfd and cargo-watch/watchexec use one of these:

```
systemfd --no-pid -s http::3030 -- cargo watch -x run
systemfd --no-pid -s http::3030 -- watchexec -r -- cargo run
```
