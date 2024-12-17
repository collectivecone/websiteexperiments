use std::{
    collections::HashMap,
    net::TcpStream,
    fs,
    io::Write,
};

#[derive(Debug, PartialEq, Eq)]
pub enum HttpTypes {
   Post,
   Get
}
#[derive(Debug)]
pub struct RequestType {
   pub http_type: HttpTypes,
   pub request: String,
}

#[derive(Debug)]
pub struct Request {
    pub request: RequestType,
    pub body: String,
    pub headers: HashMap<String,String>,
}

pub fn reply_to_get(mut stream: TcpStream,linker: &str) {
    let status_line = "HTTP/1.1 200 OK";
    let contents = fs::read_to_string(linker).unwrap();
    let length = contents.len();
    
    let response =
    format!("{status_line}\r\nContent-Length: {length}\r\n\r\n{contents}");
    stream.write_all(response.as_bytes()).unwrap();
}