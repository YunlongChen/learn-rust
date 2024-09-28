use http_server::server::HttpServer;

fn main() {
    println!("测试一个Http Server");
    // 创建线程池
    let server = HttpServer::new("127.0.0.1:10086", 4);
    server.start();
    println!("Shutting down.");
}
