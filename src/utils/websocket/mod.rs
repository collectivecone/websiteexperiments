use std::{
    sync::MutexGuard,
    net::TcpStream,
    collections::HashMap,
    ops::DerefMut
};
use tungstenite::{
    WebSocket,
    accept_with_config,
    protocol::WebSocketConfig
};
use fastrand;

#[derive(Debug)]
pub struct WebsocketData {
    pub msg: tungstenite::Message,
    pub user_id: u64,
}
#[derive(Debug)]
pub struct User {
    pub websocket: WebSocket<TcpStream>,
    pub true_ip: String,
    pub id: u64,
}


pub const STAND_WEB_CONFIG: WebSocketConfig = WebSocketConfig{
    max_send_queue: None,
    write_buffer_size: 0,
    max_write_buffer_size: usize::MAX,
    max_message_size: None,
    max_frame_size: None,
    accept_unmasked_frames: false,
};

fn get_ip(headers: &HashMap<String,String>, stream : &TcpStream) -> Option<String> {
    let ip = headers.get("CF-Connecting-IP");
    if let Some(ip) = ip {
        return Some(ip.clone());
    } else {
        match  stream.local_addr() {
            Ok(ip) =>   return  Some(ip.ip().to_string()),
            Err(_) => return None
        }
    }
}

fn is_multi_connecting(user_vec: &Vec<User>, ip_string: &String) -> bool {
    for user in user_vec {
        if user.true_ip == *ip_string {
            return true
        }
    }
    return false
}

pub fn add_new_user(stream: TcpStream,headers: HashMap<String,String>,mut guard: MutexGuard<Vec<User>>)  {
    let user_vec = guard.deref_mut();

    if let Some(ip_string) = get_ip(&headers,&stream) {
        if !is_multi_connecting(&user_vec,&ip_string) {
            _ = stream.set_nonblocking(true);
            let websocket = accept_with_config(stream,Some(STAND_WEB_CONFIG)).unwrap();
            //initalise_data(&mut websocket);
            //send_inital_monitor_data(&mut websocket);

            let user = User{
                websocket: websocket,
                true_ip: ip_string,
                id: fastrand::u64(..),
            };
            guard.push(user);
        }
    };
}

pub fn get_user_by_id(users: &mut Vec<User>, id: u64 ) -> Option<&mut User> {
    for user in users {
        if user.id == id {
            return Some(user)
        };
    };
    None
}

pub fn send_to_all_users(user_vec: &mut Vec<User>, msg: tungstenite::Message) {
    for user in user_vec.iter_mut() {
        _ = user.websocket.send(msg.clone());
    }
}

