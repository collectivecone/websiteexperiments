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
    let mut contents = fs::read(linker).unwrap();
    let length = contents.len();
  

    let data_type = linker.split(".").into_iter().last().unwrap() ;

    let header_string: String;
    if data_type == "png" {
        header_string = format!("{status_line}\r\nContent-Length: {length}\r\nContent-Type: image/png\r\n\r\n");
    } else { // data_type == "html"
         header_string = format!("{status_line}\r\nContent-Length: {length}\r\n\r\n");
    }

    
    let mut header_bytes: Vec<u8> = header_string.bytes().collect();

    header_bytes.append(&mut contents);
    stream.write_all(&*header_bytes).unwrap();
}