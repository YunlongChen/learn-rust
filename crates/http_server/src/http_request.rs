use std::collections::HashMap;

#[derive(Debug, PartialEq, Clone)]
pub enum Method {
    GET,
    POST,
    Uninitialized,
}

impl From<&str> for Method {
    fn from(value: &str) -> Self {
        match value {
            "GET" => Method::GET,
            "POST" => Method::POST,
            _ => Method::Uninitialized,
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum Resource {
    PATH(String)
}

#[derive(Debug, PartialEq)]
pub struct HttpRequest {
    method: Method,
    version: Version,
    resource: Resource,
    headers: HashMap<String, String>,
    body: Option<String>,
}

impl HttpRequest {
    pub fn new(method: Method, version: Version, resource: Resource, headers: HashMap<String, String>, body: Option<String>) -> Self {
        HttpRequest {
            method,
            version,
            resource,
            headers,
            body,
        }
    }

    pub fn method(&self) -> &Method {
        &self.method
    }

    pub fn version(&self) -> &Version {
        &self.version
    }

    pub fn resource(&self) -> &Resource {
        &self.resource
    }

    pub fn headers(&self) -> &HashMap<String, String> {
        &self.headers
    }

    pub fn body(&self) -> &Option<String> {
        &self.body
    }
}

impl From<String> for HttpRequest {
    fn from(value: String) -> Self {
        println!("request: \n{}", value);

        let mut lines = value.lines();

        let (method, resource, version) = process_req_line(lines.next().unwrap());

        let mut line_value = lines.next().unwrap();

        let mut headers: HashMap<String, String> = HashMap::new();

        while line_value.len() != 0 {
            let (key, value) = process_header_line(line_value);
            headers.insert(key, value);
            line_value = lines.next().unwrap();
        }

        let body = lines.next().map(|str| -> String { str.to_string() });

        HttpRequest {
            method,
            version,
            resource,
            headers,
            body,
        }
    }
}

fn process_req_line(line: &str) -> (Method, Resource, Version) {
    let mut parts = line.split_whitespace();

    let method = parts.next().unwrap().into();
    let resource = parts.next().unwrap().to_string();
    let version = parts.next().unwrap().into();
    (
        method,
        Resource::PATH(resource),
        version
    )
}

///
/// GET / HTTP/1.1
/// 解析请求行
///
fn process_header_line(line: &str) -> (String, String) {
    let mut header_items = line.split(":");
    let mut key = String::new();
    let mut value = String::new();
    if let Some(k) = header_items.next() {
        key = k.to_string();
    }
    if let Some(v) = header_items.next() {
        value = v.to_string();
    }
    (key, value)
}

#[derive(Debug, PartialEq)]
pub enum Version {
    V1_1,
    V2_0,
    Uninitialized,
}

impl From<&str> for Version {
    fn from(value: &str) -> Self {
        match value {
            "HTTP/1.1" => Version::V1_1,
            "HTTP/2.0" => Version::V2_0,
            _ => Version::Uninitialized,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_method() {
        let method = Method::from("GET");
        assert_eq!(method, Method::GET);

        let method: Method = "POST".into();
        assert_eq!(method, Method::POST);
    }

    #[test]
    fn test_version_from_method() {
        let test_version: Version = Version::from("HTTP/1.1");
        assert_eq!(test_version, Version::V1_1);

        let version: Version = "HTTP/1.1".into();
        assert_eq!(version, Version::V1_1);
    }

    #[test]
    fn test_http_request_from_string() {
        let request = String::from("GET / HTTP/1.1\r\nHost: localhost:3000\r\nConnection: keep-alive\r\nCache-Control: max-age=0\r\nUpgrade-Insecure-Requests: 1\r\nUser-Agent: Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/115.0.0.0 Safari/537.36\r\nAccept: text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8\r\n\r\nusername=123123");

        let http_request: HttpRequest = request.into();
        assert_eq!(http_request.method, Method::GET);
        assert_eq!(http_request.version, Version::V1_1);
        assert_eq!(http_request.resource, Resource::PATH("/".to_string()));
        assert_eq!(http_request.body, Some(String::from("username=123123")));
    }
}