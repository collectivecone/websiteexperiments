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
use crate::utils::{http::{HttpTypes,Request}, websocket::send_to_all_users};
use crate::utils::websocket::{User,add_new_user,WebsocketData,get_user_by_id};

static USERS: Mutex<Vec<User>> = Mutex::new(Vec::new());
static WEBSOCKET_SENDER: OnceLock<Sender<WebsocketData>> = OnceLock::new();


pub fn main() {
    let (websocket_sender, websocket_receiver) = mpsc::channel();
    _=WEBSOCKET_SENDER.set(websocket_sender.clone());

    spawn(move || {
        spawn(move || {
            loop {
                sleep(Duration::from_secs_f32(0.004));
                let mut guard= USERS.lock().unwrap();
                let mut users = guard.deref_mut();
                for user in users {
                    loop {
                        match user.websocket.read() {
                           Ok(msg) => {
                              _ = websocket_sender.send(WebsocketData{msg: msg,user_id: user.id});
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

                if let Ok(string) = websocket_data.msg.into_text() {

                    

                    let mut guard= USERS.lock().unwrap();
                    let mut users = guard.deref_mut();
                    let user = get_user_by_id(&mut users,websocket_data.user_id).unwrap();
                    let ip = user.true_ip.clone();

                    let formated_string = format!("{} by {}", string, user.true_ip);
    
                    send_to_all_users(users,tungstenite::Message::text(formated_string));

                }

               
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