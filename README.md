# DEPRECATED
PollWS is deprecated in favor of PollNet (https://github.com/probable-basilisk/pollnet),
which provides better functionality (in particular: secure websockets, a websocket server,
and an HTTP server) with a mostly similar API.

# pollws
Single DLL, C api, polling-based websocket client. Basically 
a wrapper to the "ws" Rust library.

## Warning
This is probably the worst piece of Rust code ever written.

## Building
Assuming you're on a 64 bit machine,
building for 64 bit:
```
cargo build --release
```

However to use this from a 32 bit Windows binary (e.g., Noita),
you'll need to build for 32 bit:
```
rustup target add i686-pc-windows-msvc
cargo build --target=i686-pc-windows-msvc --release
```

The resulting .dll will end up in `target/i686-pc-windows-msvc/release/pollws.dll`.
