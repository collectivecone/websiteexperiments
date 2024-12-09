use std::sync::{Mutex, OnceLock};

pub enum MessageType {
    User,
    System,
}

pub struct Message {
    pub text: String,
    pub by: String,
    pub message_type: MessageType,
    pub time: time::Instant,
}

pub struct Rule  {
    pub name: String,
    pub desc: String,
    pub weight: f32,
    pub process: fn(Message,User,Vec<Message>) -> Message,
}

static GLOBAL_RULES: Mutex<Vec<Rule>> = Mutex::new(Vec::new());

fn initalise_rules() {
    let guard  = GLOBAL_RULES.lock().unwrap();
    let rules = *guard;

    rules.push(Rule{ 
        name: String::from("TestRule1"), 
        desc: String::from("All the things"),
        weight: 32,
        process: |msg, user, msg_hist|  {
            msg.text.push_str(" test rule 1");
            return msg;
        }   
    });

    rules.push(Rule{ 
        name: String::from("TestRule2"), 
        desc: String::from("All the things but more"),
        weight: 32,
        process: |msg, user, msg_hist|  {
            msg.text.push_str(" test rule 2, with more rule");
            return msg;
        }   
    });

    rules.push(Rule{ 
        name: String::from("Reversed"), 
        desc: String::from("A real rule for once"),
        weight: 32,
        process: |msg, user, msg_hist|  {
            let reversed_message = msg.text.chars().rev().collect::<String>();
            msg.text = reversed_message;
            return msg;
        }   
    });
}