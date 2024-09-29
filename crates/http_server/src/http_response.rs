use std::collections::HashMap;
use std::io::Write;

#[derive(Debug, PartialEq, Clone)]
pub struct HttpResponse<'a> {
    version: &'a str,
    status_code: &'a str,
    status_text: &'a str,
    headers: Option<HashMap<&'a str, &'a str>>,
    // String这个类表示持有所有权的对象，而不是引用，如果是持有其他对象的引用，这里也需要加上生命周期表示
    body: Option<String>,
}


impl<'a> Default for HttpResponse<'a> {
    fn default() -> Self {
        HttpResponse {
            version: "HTTP/1.1".into(),
            status_code: "200".into(),
            status_text: "OK".into(),
            headers: None,
            body: None,
        }
    }
}

impl<'a> From<HttpResponse<'a>> for String {
    fn from(value: HttpResponse<'a>) -> Self {
        let mut response_string = format!("{} {} {}\r\n", value.version, value.status_code, value.status_text);
        println!("response_string: {}", response_string);
        if let Some(headers) = value.headers {
            for (key, value) in headers {
                response_string.push_str(&format!("{}: {}\r\n", key, value));
            }
        }
        if let Some(body) = value.body {
            response_string.push_str(&format!("Content-Length: {}\r\n\r\n{}\r\n", body.as_bytes().len(), body));
        } else {
            response_string.push_str(&format!("Content-Length: {}\r\n\r\n{}\r\n", 0, ""));
            println!("响应没有Body")
        }
        response_string
    }
}

impl<'a> HttpResponse<'a> {
    pub fn new(status: &'a str, headers: Option<HashMap<&'a str, &'a str>>, body: Option<String>) -> Self {
        let mut response = HttpResponse::default();

        if status != "200" {
            response.status_code = status.into();
        }

        response.headers = match &headers {
            Some(_h) => headers,
            None => {
                None
            }
        };

        HttpResponse {
            version: "HTTP/1.1".into(),
            status_code: status,
            status_text: "OK".into(),
            headers: response.headers,
            body,
        }
    }

    ///
    /// 下发响应内容
    ///
    pub fn send_response(&self, write_stream: &mut impl Write) -> std::io::Result<()> {
        let res: HttpResponse = self.clone();
        let response_string = String::from(res);
        println!("response_string: {}", response_string);
        write!(write_stream, "{}", response_string).expect(" 下发响应异常");
        Ok(())
    }

    pub fn version(&self) -> &str {
        self.version
    }

    pub fn status_code(&self) -> &str {
        self.status_code
    }

    pub fn status_text(&self) -> &str {
        self.status_text
    }

    pub fn headers(&self) -> String {
        let map: HashMap<&str, &str> = self.headers.clone().unwrap();
        let mut header_string: String = "".into();
        for (k, v) in map.iter() {
            header_string = format!("{}{}: {}\r\n", header_string, k, v);
        }
        header_string
    }

    pub fn body(&self) -> &str {
        match &self.body {
            None => {
                ""
            }
            Some(b) => {
                b.as_str()
            }
        }
    }
}
