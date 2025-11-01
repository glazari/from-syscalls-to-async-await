#![allow(bad_style)]
#![allow(unused)]
mod macros;
mod non_blocking_epoll;
mod non_blocking_mio;
mod non_blocking_poll;
mod non_blocking_select;
mod non_blocking_std;
mod sequential;
mod sequential_std;
mod sys_libc;

const CYAN: &str = "\x1b[1;36m"; //bold cyan
const PURPLE: &str = "\x1b[1;35m"; //bold purple
const RESET: &str = "\x1b[0m";

fn help() {
    let msg = format!(
        r#"{CYAN}
Usage: cargo run --bin raw-syscall -- <command>{RESET}

raw-syscall explores making TCP calls from Rust using linux libc calls directly.

{CYAN}Commands:{RESET}
    - {PURPLE}seq-calls{RESET}: Make TCP calls sequentially using blocking syscalls.
    - {PURPLE}non-blocking-select{RESET}: Make TCP calls using non-blocking sockets and select().
    - {PURPLE}non-blocking-poll{RESET}: Make TCP calls using non-blocking sockets and poll().
    - {PURPLE}non-blocking-epoll{RESET}: Make TCP calls using non-blocking sockets and epoll().
    - {PURPLE}std-seq-calls{RESET}: Make TCP calls sequentially using Rust std library.
    - {PURPLE}std-non-blocking-calls{RESET}: Make TCP calls using non-blocking sockets and std library.
    - {PURPLE}mio-non-blocking-calls{RESET}: Make TCP calls using non-blocking sockets and mio crate.
    "#,
    );
    println!("{}", msg);
}

fn main() -> Result<(), anyhow::Error> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        help();
        return Ok(());
    }
    let command = &args[1];
    match command.as_str() {
        "seq-calls" => sequential::sequential_calls()?,
        "non-blocking-select" => non_blocking_select::non_blocking_calls()?,
        "non-blocking-poll" => non_blocking_poll::non_blocking_calls()?,
        "non-blocking-epoll" => non_blocking_epoll::non_blocking_calls()?,
        "std-seq-calls" => sequential_std::sequential_calls()?,
        "std-non-blocking-calls" => non_blocking_std::non_blocking_call()?,
        "mio-non-blocking-calls" => non_blocking_mio::non_blocking_calls()?,
        _ => help(),
    }

    Ok(())
}
