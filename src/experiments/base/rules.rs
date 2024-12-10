#![allow(dead_code)]

use std::{
    ops::DerefMut, sync::Mutex, time
};
use crate::utils::websocket::User;

#[derive(Clone,Copy,Debug)]
pub enum MessageType {
    User = 0,
    System = 1,
}
#[derive(Clone,Debug)]
pub struct Message {
    pub text: String,
    pub by: String,
    pub message_type: MessageType,
    pub time: time::Instant,
}

#[derive(Debug)]
pub struct Rule  {
    pub name: String,
    pub desc: String,
    pub weight: f32,
    pub process: fn(Message,&User,&Vec<Message>) -> Message,
}

pub static GLOBAL_RULES: Mutex<Vec<Rule>> = Mutex::new(Vec::new());

pub fn initalise_rules() {
    let mut guard  = GLOBAL_RULES.lock().unwrap();
    let rules = guard.deref_mut();


    rules.push(Rule{ 
        name: String::from("TestRule1"), 
        desc: String::from("All the things"),
        weight: 32.0,
        process: |mut msg, _, _|  {
            msg.text.push_str(" test rule 1");
            return msg;
        }   
    });

    rules.push(Rule{ 
        name: String::from("TestRule2"), 
        desc: String::from("All the things but more"),
        weight: 32.0,
        process: |mut msg, _, _|  {
            msg.text.push_str(" test rule 2, with more rule");
            return msg;
        }   
    });

    rules.push(Rule{ 
        name: String::from("Reversed"), 
        desc: String::from("A real rule for once"),
        weight: 32.0,
        process: |mut msg, _, _|  {
            let reversed_message = msg.text.chars().rev().collect::<String>();
            msg.text = reversed_message;
            return msg;
        }   
    });
}