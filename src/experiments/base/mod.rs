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
static MSGS: Mutex<Vec<Message>> = Mutex::new(Vec::new());

fn message_to_serde(msg: &Message) -> serde_json::Value {
    serde_json::json!([
        msg.text,
        msg.by,
        msg.message_type as u8,
    ])
}

fn make_message_tung(msgs: &Vec<Message>) -> tungstenite::Message {
    let mut vec: Vec<serde_json::Value> = Vec::new();
    vec.append(serde_json::Value::String(String::from("Messages")));

    for msg in msgs {
        vec.push(message_to_serde(msg));
    }

    let final_string = serde_json::Value::Array(vec);
    return tungstenite::Message::text(final_string);
}

fn current_rules_json() -> tungstenite::Message {
    let mut vec: Vec<serde_json::Value> = Vec::new();
    vec.append(serde_json::Value::String(String::from("Rules")));

    for rule in RULES.lock().unwrap().deref_mut() {
        let rule_json = serde_json::json!([
             rule.name,
             rule.desc,
             1,
             1,
        ]);
        vec.push(rule_json);
    }

    let final_string = serde_json::Value::Array(vec);
    return tungstenite::Message::text(final_string);
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

            let msg = Message{
                text: string,
                by: ip, 
                message_type: MessageType::User,
                time: time::Instant::now(),
            };

            send_to_all_users(users,make_message_tung(&Vec!(msg)) ) ;
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
    let mut guard: std::sync::MutexGuard<'_, Vec<User>> = USERS.lock().unwrap();
    let user = add_new_user(stream, request.headers, &mut guard);

    if let Some(user) = user {
        send_to_user(&mut user, current_rules_json());
        let guard = MSGS.lock().unwrap();
        let msgs = guard.deref_mut(); 
        send_to_user(&mut user, make_message_tung(msgs));
    }
}