use crate::handler::{Handler, StaticHandler, WebServiceHandler};
use crate::http_request::{HttpRequest, Method, Resource};
use std::io::Write;

#[derive(Debug)]
pub struct Router {}

impl Router {
    pub(crate) fn route(http_request: HttpRequest, tcp_stream: &mut impl Write) {
        println!("Routing request: {:?}", http_request.resource());
        match http_request.method() {
            Method::GET => match &http_request.resource() {
                Resource::PATH(path) => {
                    let route: Vec<&str> = path.split("/").collect();
                    match route[1] {
                        "api" => {
                            let response = WebServiceHandler::handle(&http_request);
                            response.send_response(tcp_stream).unwrap()
                        }
                        _ => {
                            let response = StaticHandler::handle(&http_request);
                            response.send_response(tcp_stream).unwrap()
                        }
                    }
                }
            },
            Method::POST => {}
            Method::Uninitialized => {
                // TODO: handle this error
            }
        }
    }
}
