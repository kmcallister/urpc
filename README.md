# urpc: tiny RPC framework for Rust

An RPC interface contains one or more function signatures.

```rust
urpc! {
    pub interface sort {
        fn sort(xs: Vec<u8>) -> Vec<u8> {}
    }
}
```

`urpc!` is an ordinary `macro_rules!` macro and will be available in Rust 1.0.
It generates, among other things, a trait which you can then implement:

```rust
struct LocalSort;

impl sort::Methods for LocalSort {
    fn sort(&mut self, mut xs: Vec<u8>) -> urpc::Result<Vec<u8>> {
        xs.sort();
        Ok(xs)
    }
}
```

You can build an RPC server from any implementation of `Methods` plus any
implementation of standard IO's `Read + Write`.

```rust
sort::serve(LocalSort, socket);
```

A client implements `Methods` by communicating with the server.

```rust
let mut client = sort::Client::new(socket);
client.sort(vec![8, 6, 7, 5, 3, 0, 9]).unwrap();
```

See `tests/` for full examples.  The protocol is simply [bincode][] over the
`Read + Write` stream.  This library has no knowledge of sockets or networking
per se (outside of the tests).

What this library **does not provide**: authentication, encryption, message
integrity, service discovery, protocol extensibility, particularly graceful
error handling, request pipelining, multiple concurrent clients, capabilities,
distributed objects, etc. etc.  It is perhaps a useful building block for
systems providing some of the above.

My use case for this library is failure isolation for unsafe code, a context
where the above features are not needed. `tests/server-crash.rs` shows an
example of "catching" a memory corruption crash as an `Err`, by running the
unsafe code in a child process.  Combined with [OS-level process sandboxing][],
this minimizes the security impact of vulnerable C libraries used in a Rust
program, in cases where the multi-process architecture and IPC overhead are
acceptable.

The library is in a **very** early stage and has numerous limitations, some of
which are documented on the issue tracker.

[bincode]: https://github.com/TyOverby/bincode
[OS-level process sandboxing]: https://github.com/pcwalton/gaol
