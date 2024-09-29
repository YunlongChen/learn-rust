use crate::http_request::{HttpRequest, Resource};
use crate::http_response::HttpResponse;
use std::collections::HashMap;
use std::env;
use std::fs::read_to_string;

pub trait Handler {
    ///
    /// Handle the request and return a response
    ///
    fn handle(req: &HttpRequest) -> HttpResponse;

    ///
    /// Load a file from the static directory
    ///
    fn load_file(file_name: &str) -> Option<String> {
        println!("load_file: {}", file_name);
        let default_path = format!("{}\\static\\", env!("CARGO_MANIFEST_DIR"));
        println!("default_path: {}", default_path);
        let public_path = env::var("PUBLIC_PATH").unwrap_or(default_path);

        let file_path = format!("{}{}", public_path, file_name);
        println!("Loading file: {}", file_path);
        read_to_string(file_path).ok()
    }
}


#[derive(Debug)]
pub struct StaticHandler;
#[derive(Debug)]
pub struct PageNotFoundHandler;
#[derive(Debug)]
pub struct WebServiceHandler;

impl Handler for WebServiceHandler {
    fn handle(request: &HttpRequest) -> HttpResponse {
        let Resource::PATH(s) = &request.resource();
        println!("Request: {}", s);
        let _ = Self::load_json("test.json");
        None.unwrap()
    }
}

impl Handler for PageNotFoundHandler {
    fn handle(req: &HttpRequest) -> HttpResponse {
        let Resource::PATH(s) = &req.resource();
        let route: Vec<&str> = s.split("/").collect();
        println!("Page not found: {}", route[1]);
        HttpResponse::new("404", Some(HashMap::new()), Some(String::from("Page not found")))
    }
}

impl Handler for StaticHandler {
    fn handle(req: &HttpRequest) -> HttpResponse {
        let Resource::PATH(s) = &req.resource();
        let route: Vec<&str> = s.split("/").collect();
        match route[1] {
            "" => {
                let file = Self::load_file("index.html");
                if let Some(ref file_content) = file {
                    println!("Loading file content: {}", &file_content);
                }
                let mut map: HashMap<&str, &str> = HashMap::new();
                map.insert("Content-Type", "text/html");
                HttpResponse::new("200", Some(map), file)
            }
            "healthy" => {
                let option = Self::load_file("healthy.html");
                HttpResponse::new("200", None, option)
            }
            path => match Self::load_file(path) {
                Some(content) => {
                    let mut map: HashMap<&str, &str> = HashMap::new();
                    if path.ends_with(".css") {
                        map.insert("Content-Type", "text/css");
                    } else if path.ends_with(".js") {
                        map.insert("Content-Type", "text/javascript");
                    } else { map.insert("Content-Type", "text/html"); }
                    HttpResponse::new("200", Some(map), Some(content))
                }
                None =>
                    HttpResponse::new("404", None, Self::load_file("404.html"))
            }
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct OrderStatus {
    order_id: String,
    order_date: String,
    order_status: String,
}

impl WebServiceHandler {
    fn load_json(file_name: &str) -> Vec<OrderStatus> {
        let default_path = format!("{}/static", env!("CARGO_MANIFEST_DIR"));
        let public_path = env::var("PUBLIC_PATH").unwrap_or(default_path);
        let file_path = format!("{}/{}", public_path, file_name);
        println!("Loading file: {}", file_path);

        let mut vec = Vec::new();
        vec.insert(0, OrderStatus {
            order_id: String::from("1"),
            order_date: String::from("2022-01-01"),
            order_status: String::from("Delivered"),
        });
        vec
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let handler = StaticHandler::load_file("index.html");
        if let Some(file_content) = handler {
            println!("{}", file_content);
            assert!(file_content.len() > 0);
        } else {
            assert!(false);
        }
    }
}
