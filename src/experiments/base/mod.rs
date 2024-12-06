use std::{
    fs,
    io::Write,
    net::{TcpListener,TcpStream},
    ops::DerefMut,
    sync::{
        mpsc::{self,Sender,Receiver},
        LazyLock,
        Mutex,
        OnceLock,
    },
    time::Duration,
    thread::{spawn,sleep},
};
use crate::utils::http::{HttpTypes,Request};
use crate::utils::websocket::{User,add_new_user,WebsocketData};

static USERS: Mutex<Vec<User>> = Mutex::new(Vec::new());
static WEBSOCKET_SENDER: OnceLock<Sender<WebsocketData>> = OnceLock::new();



pub fn main() {
    let (websocket_sender, websocket_receiver) = mpsc::channel();
    WEBSOCKET_SENDER.set(websocket_sender.clone());
    spawn(move || {
        spawn(move || {
            loop {
                sleep(Duration::from_secs_f32(0.01));
                let mut guard= USERS.lock().unwrap();
                let mut users = guard.deref_mut();
                for user in users {
                    loop {
                        match user.websocket.read() {
                           Ok(msg) => {
                              _ = websocket_sender.send(WebsocketData{msg: msg,user_id: 0});
                           }
                           Err(e) => {
                            if e.to_string() == "Trying to work with closed connection" {
                                println!("user should be deleted, but I dont' know what to do here!")
                            } 
                            break;
                           }
                        }
                    }
                }
            }
        });
        spawn(move || {
            for websocket_data in websocket_receiver {
                
            }
        });
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
    let mut guard: std::sync::MutexGuard<'_, Vec<User>> = USERS.lock().unwrap();
    add_new_user(stream, request.headers, guard);


}