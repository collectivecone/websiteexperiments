use std::{
    fs,
    net::{TcpListener,TcpStream},
    io::Write,
};
use crate::utils::http::{HttpTypes,Request,RequestType};

pub fn http_request(mut stream: TcpStream, request: Request) {

    match request.request.http_type {
        HttpTypes::Get => {
            let status_line = "HTTP/1.1 200 OK";
            let contents = fs::read_to_string("src/experiments/base/Website.html").unwrap();
            let length = contents.len();
            
            let response =
            format!("{status_line}\r\nContent-Length: {length}\r\n\r\n{contents}");
            stream.write_all(response.as_bytes()).unwrap();
        }
        _ => {}
    }
}

pub fn websocket_request() {



}