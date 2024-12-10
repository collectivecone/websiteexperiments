use std::{
    alloc::GlobalAlloc, net::TcpStream, ops::DerefMut, sync::{
        mpsc,
        Mutex,
    }, thread::{sleep, spawn}, time::{self, Duration}
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
        send_to_all_users,
    }
};

use serde_json;

pub mod rules;
use rules::{
    Rule,
    Message,
    MessageType,
    GLOBAL_RULES,
};

const RULE_TIME: u32 = 23;
const RULE_MAX: usize = 4;
static USERS: Mutex<Vec<User>> = Mutex::new(Vec::new());
static RULES: Mutex<Vec<Rule>> = Mutex::new(Vec::new());

fn message_to_tung(msg: Message) {

    //let time_stamp = msg.time.duration_since(earlier)

    let msg_str =  serde_json::json!([
        msg.text,
        msg.by,
        msg.message_type as u8,
    ]).to_string(); 

    return tungstenite::Message::text(msg_str);
}

fn current_rules_json() {
    let mut Vec: Vec<serde_json::Value> = Vec::new();

    for rule in RULES.lock().unwrap().deref_mut() {
        let rule_json = serde_json::json!([
             rule.name ,
             rule.desc ,
             1 ,
             1 ,
        ]);

        Vec.push(rule_json);
    }

    let final_string = serde_json::Value::Array(Vec);
    println!("{}", final_string)
}

pub fn main() {
    let (websocket_sender, websocket_receiver) = mpsc::channel();
    websocket::read_all_inputs(&USERS,websocket_sender);

    rules::initalise_rules();
    spawn(|| {
        let mut g_rules = RULES.lock().unwrap(); let mut rules: &mut Vec<Rule> = g_rules.deref_mut();
        let mut g_g_rules = GLOBAL_RULES.lock().unwrap(); let mut global_rules = g_g_rules.deref_mut();

        let g_rule = global_rules.pop().unwrap();
        rules.push(g_rule);
        drop(g_rules); drop(g_g_rules);
        current_rules_json();

        loop {
            sleep(Duration::from_secs(32));
            println!("looping")
        }
    });

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