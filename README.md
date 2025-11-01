# From Syscalls to Async/Await in Rust
Understanding the inner workings of Rust runtimes.


This repository is the code complement to the Talk given at Rust SP meetup in Sao Paulo on October 2025.
- [meetup link](https://www.meetup.com/rust-sao-paulo-meetup/events/311084440/)
- [slides link](https://docs.google.com/presentation/d/e/2PACX-1vRL_snOYPRY76nYNLnh1GKNLDqp84Z5flk2BYGPfDr6U3ZLyke6o2iXE23VowBoxKsTaPvQ_PJWcZo9/pub?start=false&loop=false&delayms=60000#slide=id.p)


## Packages

- `delay-server`: A simple Http server that delays reponses for 100ms to help test non-blocking behavior in clients.
- `raw-syscall`: explores the syscalls used to make network requests and to make them non-blocking. (actually uses libc bindings, not raw syscalls)
- `manual-futures`: implements several simple runtimes using manual futures to understand how async/await works under the hood.

### Delay Server

Delay server is only used as a counterpart to the code in the other packages. Runs with:

```bash
cargo run -p delay-server
```

### Raw Syscall

This package contains several implementations of making TCP requests using different techniques, from blocking syscalls to non-blocking syscalls with different multiplexing strategies, to Rust std library and mio crate.

```bash
cargo run -p raw-syscall -- <command>
```

Where `<command>` is one of:
- `seq-calls`: Make TCP calls sequentially using blocking syscalls.
- `non-blocking-select`: Make TCP calls using non-blocking sockets and select().
- `non-blocking-poll`: Make TCP calls using non-blocking sockets and poll().
- `non-blocking-epoll`: Make TCP calls using non-blocking sockets and epoll().
- `std-seq-calls`: Make TCP calls sequentially using Rust std library.
- `std-non-blocking-calls`: Make TCP calls using non-blocking sockets and std library.
- `mio-non-blocking-calls`: Make TCP calls using non-blocking sockets and mio crate.

### Manual futures

```bash
cargo run -p manual-futures -- <command>
```

Where `<command>` is one of:
- `naive-executor`: Polls future to completion in a loop.
- `epoll-executor`: Uses epoll to wait for readiness before polling futures again.
- `futures-executor`: Uses the futures crate executor to run our own futures as proof they work.
- `waker-executor`: Uses a custom waker and reactor to drive the futures to completion.
- `tokio-future`: Run a tokio futures based async function using our waker-executor. (requires starting tokio so that it's reactor starts)


 
