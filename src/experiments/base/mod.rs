use std::{
    fs:: 
        OpenOptions
    , io::{Read, Write}, net::TcpStream, ops::DerefMut, sync::{
        mpsc,
        Mutex,
    }, thread::{sleep,spawn},
    time:: Duration
    
};

use crate::utils::{
    self, filter, http::{
        reply_to_get, HttpTypes, Request
    }, websocket::{
        self, add_new_user, get_user_by_id, send_to_all_users, send_to_user, User
    }
};

use serde_json::{self, Error};
use fastrand;

pub mod rules;
use rules::{
    Rule,
    Message,
    MessageType,
    GLOBAL_RULES,
};

const RULE_TIME: u64 = 24;
const RULE_MAX: usize = 1;
const MAX_MSGS: usize = 1000;
static USERS: Mutex<Vec<User>> = Mutex::new(Vec::new());
static RULES: Mutex<Vec<Rule>> = Mutex::new(Vec::new());
static MSGS: Mutex<Vec<Message>> = Mutex::new(Vec::new());
static UNSAVED_MSG: Mutex<Vec<Message>> = Mutex::new(Vec::new());


fn write_msg_history() {
    let mut guard = UNSAVED_MSG.lock().unwrap();
    let unsaved_msgs = guard.deref_mut();

    let mut file = OpenOptions::new()
        .append(true)
        .create(true)
        .open("src/experiments/base/msghistory.txt")
        .unwrap();

    let mut message_list = String::new();

    for msg in &(*unsaved_msgs) {
        let msg = msg;
        let mut save = message_to_serde(msg).to_string();
        save.push('\n');
        message_list.push_str(save.as_str());
    }

    let bytes: &[u8] = message_list.as_str().as_bytes();
    _=file.write_all(bytes);

    unsaved_msgs.clear();
}

fn read_msg_history() {
    let  file_o = OpenOptions::new()
    .read(true)
    .open("src/experiments/base/msghistory.txt");

    if let Err(_) = file_o {
        return;
    }
    let mut file = file_o.unwrap();

    let mut data = String::new();

    _=file.read_to_string(&mut data);

    let mut msgs_str = data.split("\n").collect::<Vec<&str>>();
    if msgs_str.len() > MAX_MSGS {
        msgs_str = msgs_str[(msgs_str.len() - MAX_MSGS)..].to_vec();

    };

    let mut g_msgs: std::sync::MutexGuard<'_, Vec<Message>> = MSGS.lock().unwrap(); let msgs = g_msgs.deref_mut(); 

    for msg in msgs_str {
        let mut a = String::from("");
        a.push_str(msg);

        let json: Result<serde_json::Value, Error> = serde_json::from_str(msg);
        if let Ok(okayness) = json {
            if let serde_json::Value::Array(array) = okayness {
                msgs.push(Message{
                    text: String::from(array[0].as_str().unwrap()),
                    by: String::from(array[1].as_str().unwrap()),
                    message_type: {
                        let num = array[2].as_u64().unwrap();
                        if num == 0 {MessageType::User} else {MessageType::System}
                    },
                    time: array[3].as_u64().unwrap(),
                })
            }
        }
    }
}

fn add_to_msg_history(msg: &mut Message, msgs: &mut Vec<Message>) {
    msgs.push(msg.clone());
    let mut guard = UNSAVED_MSG.lock().unwrap();
    let unsaved_msg = guard.deref_mut();
    unsaved_msg.push(msg.clone());
}

fn message_to_serde(msg: &Message) -> serde_json::Value {
    serde_json::json!([
        msg.text.clone(),
        msg.by.clone(),
        msg.message_type as u8,
        msg.time,
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
             rule._starttime,
             rule._endtime,
        ]);
        vec.push(rule_json);
    }
    
    let final_string = serde_json::Value::Array(vec).to_string();
    return tungstenite::Message::text(final_string);
}

pub fn main() {
    let (websocket_sender, websocket_receiver) = mpsc::channel();
    websocket::read_all_inputs(&USERS,websocket_sender);

    read_msg_history();
    rules::initalise_rules();
    spawn(|| {
        loop {
    
            let mut g_rules = RULES.lock().unwrap(); let rules: &mut Vec<Rule> = g_rules.deref_mut();
            let mut g_g_rules = GLOBAL_RULES.lock().unwrap(); let global_rules = g_g_rules.deref_mut();
           
        
            let mut g_msgs: std::sync::MutexGuard<'_, Vec<Message>> = MSGS.lock().unwrap(); let mut msgs = g_msgs.deref_mut(); 
            let mut guard= USERS.lock().unwrap(); let users: &mut Vec<User> = guard.deref_mut();
            if users.len() > 0 {
                let mut g_rule = global_rules.remove(fastrand::usize(..global_rules.len()));
                g_rule._starttime = utils::unix_time(); g_rule._endtime = utils::unix_time() + 1000 * (RULE_TIME * RULE_MAX as u64) ;
                let rule_name = g_rule.name.clone();
                rules.push(g_rule);
                if rules.len() > RULE_MAX {
                    let rule = rules.remove(0);
                    let mut system_message = Message{
                        text: format!("{} has been replaced by {}", rule_name, rule.name.clone()),
                        time: utils::unix_time(),
                        message_type: MessageType::System,
                        by: String::from("server"),
    
                    };
                    add_to_msg_history(&mut system_message,&mut msgs);
                    send_to_all_users(users,make_message_tung(&vec!(system_message)));
    
                    global_rules.push(rule);
                    
                }
            }
         
            drop(g_rules); drop(g_g_rules); drop(g_msgs);
        
          
            send_to_all_users(users,current_rules_json());

            drop(guard);
            
            write_msg_history();
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
                time: utils::unix_time(),
            };

            let mut g_msgs: std::sync::MutexGuard<'_, Vec<Message>> = MSGS.lock().unwrap(); let mut msgs = g_msgs.deref_mut(); 
            let mut g_rules = RULES.lock().unwrap(); let rules: &mut Vec<Rule> = g_rules.deref_mut();
            msg.text.truncate(300);
            for rule in rules {
                msg = (rule.process)(msg,&user,&msgs)
            }

            msg.text = filter::censore_message(msg.text);
            
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