use std::io::{Read};
use std::net::{TcpListener, TcpStream};
use log::info;
use crate::http_request::HttpRequest;
use crate::router::Router;
use crate::thread_pool::ThreadPool;

#[derive(Debug)]
pub struct HttpServer {
    // 创建线程池
    thread_pool: ThreadPool,
    socket_addr: String,
}

impl HttpServer {
    pub fn new(addr: &str, thread_count: usize) -> HttpServer {
        // 创建线程池
        let thread_pool = ThreadPool::new(thread_count);

        HttpServer {
            thread_pool,
            socket_addr: addr.to_string(),
        }
    }

    pub fn start(&self) {
        let listener = TcpListener::bind(&self.socket_addr).expect("绑定到端口失败");
        println!("服务器启动，监听地址：{}", self.socket_addr);
        for incoming in listener.incoming() {
            let stream = incoming.unwrap();
            println!("连接已建立");
            self.thread_pool.execute(|| handle_connection(stream));
        }
    }
}

fn handle_connection(mut stream: TcpStream) {
    info!("收到http请求！");

    let mut buf = [0; 1024];
    stream.read(&mut buf).expect("读取内容发生了异常");

    let request_str = String::from_utf8(buf.to_vec()).expect("读取内容发生了异常");

    // println!("读取了{}个字节, 请求内容：{}", request_str.len(), request_str);

    let request: HttpRequest = request_str.into();
    Router::route(request, &mut stream);
    // let request_line = buf_reader.lines().next().unwrap().unwrap();
    //
    // let (status_line, filename) = match &request_line[..] {
    //     "GET / HTTP/1.1" => ("HTTP/1.1 200 OK", "hello.html"),
    //     "GET /sleep HTTP/1.1" => ("HTTP/1.1 200 OK", "hello.html"),
    //     _ => ("HTTP/1.1 404 NOT FOUND", "hello.html"),
    // };
    //
    // let contents = fs::read_to_string(filename).unwrap();
    // let length = contents.len();
    // let response: String = "HTTP/1.1 200 OK\r\nContent-Length: 0\r\n\r\n".into();
    // println!("请求处理完成");
    // let result = stream.write_all(response.as_bytes()).unwrap();
    // println!("发送了{:?}个字节", result);
}