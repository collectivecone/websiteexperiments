use std::{
    fs, io::Write, net::{TcpListener,TcpStream}, sync::Mutex
};
use crate::utils::http::{HttpTypes,Request};
use crate::utils::websocket::{User,add_new_user};


static USERS: Mutex<Vec<User>> = Mutex::new(Vec::new());

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

pub fn websocket_request(mut stream: TcpStream, request: Request) {
    let guard: std::sync::MutexGuard<'_, Vec<User>> = USERS.lock().unwrap();
    add_new_user(stream, request.headers, guard);


}