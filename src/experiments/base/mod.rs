use std::{
    net::TcpStream, ops::DerefMut, sync::{
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
        get_user_by_id,
        send_to_all_users,
        send_to_user,
    }
};

use serde_json;
use fastrand;

pub mod rules;
use rules::{
    Rule,
    Message,
    MessageType,
    GLOBAL_RULES,
};

const RULE_TIME: u64 = 24;
const RULE_MAX: usize = 2;
static USERS: Mutex<Vec<User>> = Mutex::new(Vec::new());
static RULES: Mutex<Vec<Rule>> = Mutex::new(Vec::new());
static MSGS: Mutex<Vec<Message>> = Mutex::new(Vec::new());

fn add_to_msg_history(msg: &mut Message, msgs: &mut Vec<Message>) {
    msgs.push(msg.clone());
}

fn message_to_serde(msg: &Message) -> serde_json::Value {
    serde_json::json!([
        msg.text.clone(),
        msg.by.clone(),
        msg.message_type as u8,
    ])
}

fn make_message_tung(msgs: &Vec<Message>) -> tungstenite::Message {
    let mut vec: Vec<serde_json::Value> = Vec::new(); 
    vec.push(serde_json::Value::String(String::from("Messages")));

    for msg in msgs {
        vec.push(message_to_serde(msg));
    }

    let final_string = serde_json::Value::Array(vec).to_string();
    return tungstenite::Message::text(final_string);
}

fn current_rules_json() -> tungstenite::Message {
    let mut vec: Vec<serde_json::Value> = Vec::new();
    vec.push(serde_json::Value::String(String::from("Rules")));

    let mut guard = RULES.lock().unwrap();
    for rule in guard.deref_mut() {
        let rule_json = serde_json::json!([
             rule.name,
             rule.desc,
             1,
             1,
        ]);
        vec.push(rule_json);
    }

    let final_string = serde_json::Value::Array(vec).to_string();
    return tungstenite::Message::text(final_string);
}

pub fn main() {
    let (websocket_sender, websocket_receiver) = mpsc::channel();
    websocket::read_all_inputs(&USERS,websocket_sender);

    rules::initalise_rules();
    spawn(|| {
        loop {
            let mut g_rules = RULES.lock().unwrap(); let rules: &mut Vec<Rule> = g_rules.deref_mut();
            let mut g_g_rules = GLOBAL_RULES.lock().unwrap(); let global_rules = g_g_rules.deref_mut();
            let g_rule = global_rules.remove(fastrand::usize(..global_rules.len()));
            rules.push(g_rule);
            if rules.len() > RULE_MAX {
                let rule = rules.remove(0);
                global_rules.push(rule);
            }

            drop(g_rules); drop(g_g_rules);
        
            let mut guard= USERS.lock().unwrap(); let users: &mut Vec<User> = guard.deref_mut();
            send_to_all_users(users,current_rules_json());
            drop(guard);
            sleep(Duration::from_secs(RULE_TIME));
        }
    });

    for websocket_data in websocket_receiver {
        if let Ok(string) = websocket_data.msg.into_text() {
            let mut guard= USERS.lock().unwrap();
            let mut users: &mut Vec<User> = guard.deref_mut();
            let user: &mut User = get_user_by_id(&mut users,websocket_data.user_id).unwrap();
            let ip = user.true_ip.clone();

            let mut msg = Message{
                text: string,
                by: ip, 
                message_type: MessageType::User,
                time: time::Instant::now(),
            };

            let mut g_msgs: std::sync::MutexGuard<'_, Vec<Message>> = MSGS.lock().unwrap(); let mut msgs = g_msgs.deref_mut(); 
            let mut g_rules = RULES.lock().unwrap(); let rules: &mut Vec<Rule> = g_rules.deref_mut();
            for rule in rules {
                msg = (rule.process)(msg,&user,&msgs)
            }
            
            add_to_msg_history(&mut msg,&mut msgs);
            send_to_all_users(users,make_message_tung(&vec!(msg)) ) ;
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
    let user_op: Option<&mut User> = add_new_user(stream, request.headers, &mut guard);

    if let Some(user) = user_op {
        send_to_user(user, current_rules_json());
        let mut guard = MSGS.lock().unwrap();
        let msgs = guard.deref_mut(); 
        send_to_user(user, make_message_tung(msgs));
    }
}