use std::{
    fs, io::Write, net::TcpStream, 
    ops::DerefMut,
    sync::{
        mpsc::{self, channel, Receiver, Sender},
        Mutex,
        LazyLock
    },
    thread::{
        sleep, spawn
    }, time::Duration
};
use crate::utils::http::{HttpTypes,Request};
use crate::utils::websocket::{User,add_new_user};

static USERS: Mutex<Vec<User>> = Mutex::new(Vec::new());
static WEBSOCKET_SENDER: Mutex<Option<Sender<String>>> = Mutex::new(None);



pub fn main() {
    let (websocket_sender,websocket_receiver) = mpsc::channel();
    let guard = WEBSOCKET_SENDER.lock().unwrap();
    let option = guard.deref_mut();
    option = &mut Some(websocket_sender);
    drop(guard);

    spawn(move || {
        loop {
            sleep(Duration::from_secs_f32(0.1));
            let mut guard = USERS.lock().unwrap();
            let mut users =  guard.deref_mut();
            for user in users {
                
            }
            drop(guard);
        }
    });
}

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
    println!("yes");
    let mut guard: std::sync::MutexGuard<'_, Vec<User>> = USERS.lock().unwrap();
    add_new_user(stream, request.headers, guard);
}