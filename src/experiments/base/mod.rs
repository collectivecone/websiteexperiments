use std::{
    fs,
    io::Write,
    net::TcpStream,
    ops::DerefMut,
    sync::{
        mpsc::{self,Sender},
        Mutex,
        OnceLock,
    },
    thread::spawn,
};
use crate::utils::{
    http::{
        HttpTypes,
        Request,
        reply_to_get,
    }, 
    websocket::{
        self,
        User,
        add_new_user,
        WebsocketData,
        get_user_by_id,
        send_to_all_users
    }
};

static USERS: Mutex<Vec<User>> = Mutex::new(Vec::new());

pub fn main() {
    let (websocket_sender, websocket_receiver) = mpsc::channel();
    websocket::read_all_inputs(&USERS,websocket_sender);

    for websocket_data in websocket_receiver {
        if let Ok(string) = websocket_data.msg.into_text() {
            let mut guard= USERS.lock().unwrap();
            let mut users: &mut Vec<User> = guard.deref_mut();
            let user: &mut User = get_user_by_id(&mut users,websocket_data.user_id).unwrap();
            let ip = user.true_ip.clone();

            let formated_string = format!("{} by {}", string, ip);

            send_to_all_users(users,tungstenite::Message::text(formated_string));
        }
    }
}

pub fn http_request(stream: TcpStream, request: Request) {
    match request.request.http_type {
        HttpTypes::Get => reply_to_get(stream,"src/experiments/base/Website.html"),
        _ => {}
    }
}

pub fn websocket_request(stream: TcpStream, request: Request) {
    let guard: std::sync::MutexGuard<'_, Vec<User>> = USERS.lock().unwrap();
    add_new_user(stream, request.headers, guard);
}