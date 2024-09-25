use std::fs;
use std::io::{BufRead, BufReader, Write};
use http_server::ThreadPool;
use std::net::{TcpListener, TcpStream};
use log::info;

fn main() {

    println!("这里是一个测试功能");
    // 创建线程池
    let thread_pool = ThreadPool::new(4);
    // 创建一个服务监听
    let listener = TcpListener::bind("127.0.0.1:10086").unwrap();
    for incoming in listener.incoming() {
        let stream = incoming.unwrap();
        thread_pool.execute(|| handle_connection(stream));
    }
    println!("Shutting down.");
}

fn handle_connection(mut stream: TcpStream) {
    info!("收到http请求！");
    let buf_reader = BufReader::new(&mut stream);
    let request_line = buf_reader.lines().next().unwrap().unwrap();

    let (status_line, filename) = match &request_line[..] {
        "GET / HTTP/1.1" => ("HTTP/1.1 200 OK", "hello.html"),
        "GET /sleep HTTP/1.1" => ("HTTP/1.1 200 OK", "hello.html"),
        _ => ("HTTP/1.1 404 NOT FOUND", "hello.html"),
    };

    let contents = fs::read_to_string(filename).unwrap();
    let length = contents.len();
    let response =
        format!("{status_line}\r\nContent-Length: {length}\r\n\r\n{contents}");
    stream.write_all(response.as_bytes()).unwrap();
}