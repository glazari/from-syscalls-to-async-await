use std::thread;

use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;

mod connect;
mod connect_mio;
mod epoll_executor;
mod executor_naive;
mod receive;
mod receive_mio;
mod send;
mod send_mio;
mod waker;
mod waker_connect;
mod waker_executor;
mod waker_reactor;
mod waker_receive;
mod waker_send;

const CYAN: &str = "\x1b[1;36m"; //bold cyan
const PURPLE: &str = "\x1b[1;35m"; //bold purple
const RESET: &str = "\x1b[0m";

const REQUEST: &str = "GET / HTTP/1.1
Host: localhost
Connection: close

";

fn help() {
    let msg = format!(
        r#"{CYAN}
Usage: cargo run --bin manual-futures -- <command>{RESET}

manual-futures explores making very simple async executors and reactors from scratch.

{CYAN}Commands:{RESET}
    - {PURPLE}naive-executor{RESET}: Polls future to completion in a loop.
    - {PURPLE}epoll-executor{RESET}: Uses epoll to wait for readiness before polling again.
    - {PURPLE}futures-executor{RESET}: Uses the futures crate executor to run our own futures as proof they work.
    - {PURPLE}waker-executor{RESET}: Uses a custom waker and reactor to drive the futures to completion.
     "#,
    );
    println!("{}", msg);
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        help();
        return;
    }

    let command = &args[1];
    match command.as_str() {
        "naive-executor" => executor_naive::block_on(async_main()),
        "epoll-executor" => epoll_executor::block_on(async_main_mio()),
        "futures-executor" => futures::executor::block_on(async_main()),
        "waker-executor" => {
            thread::spawn(waker_reactor::run_reactor);
            thread::sleep(std::time::Duration::from_millis(100)); // Give the reactor some time to start
            waker_executor::block_on(async_main_waker());
        }
        "tokio-future" => {
            // Creates a Tokio runtime so that we initialize tokio's reactor
            let rt = tokio::runtime::Runtime::new().unwrap();
            let _guard = rt.enter(); // Enter the runtime context (this is checked by Tokio)

            // use our waker executor to run a tokio future
            waker_executor::block_on(tokio_async_main());
        }
        _ => help(),
    }
}

async fn async_main() {
    let mut stream = connect::connect_async("127.0.0.1:3000").await;
    println!("Connected to server");
    let _ = send::send_async(&mut stream, REQUEST).await;
    println!("Request sent");
    let response = receive::receive_async(&mut stream).await;
    println!("Response received");
    let response_str = String::from_utf8_lossy(&response);
    println!("Response:\n{}", response_str);
}

async fn async_main_mio() {
    let mut mio_stream = connect_mio::connect_async_mio("127.0.0.1:3000").await;
    println!("Connected to server (mio)");
    let _ = send_mio::send_async(&mut mio_stream, REQUEST).await;
    println!("Request sent (mio)");
    let response = receive_mio::receive_async(&mut mio_stream).await;
    println!("Response received (mio)");
    let response_str = String::from_utf8_lossy(&response);
    println!("Response:\n{}", response_str);
}

async fn async_main_waker() {
    let mut stream = waker_connect::connect_async("127.0.0.1:3000").await;
    println!("Connected to server (waker)");
    let _ = waker_send::send_async(&mut stream, REQUEST).await;
    println!("Request sent (waker)");
    let response = waker_receive::receive_async(&mut stream).await;
    println!("Response received (waker)");
    let response_str = String::from_utf8_lossy(&response);
    println!("Response:\n{}", response_str);
}

async fn tokio_async_main() {
    let mut stream = TcpStream::connect("127.0.0.1:3000").await.unwrap();
    println!("Connected to server (tokio-future)");
    stream.write_all(REQUEST.as_bytes()).await.unwrap();
    println!("Request sent (tokio-future)");
    let mut response = Vec::new();
    stream.read_to_end(&mut response).await.unwrap();
    println!("Response received (tokio-future)");
    let response_str = String::from_utf8_lossy(&response);
    println!("Response:\n{}", response_str);
}
