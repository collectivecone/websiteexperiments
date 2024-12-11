#![allow(dead_code)]

use std::{
    ops::DerefMut, sync::Mutex, time
};
use crate::utils::websocket::User;
use fastrand;

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

#[derive(Clone,Debug)]
enum Section {
    Word(String),
    SpaceOrPunc(String),
}
use Section::{Word,SpaceOrPunc};

fn combine_section_vec_into_string(msg_section: Vec<Section>) -> String {
    let mut msg = String::new();
    for sect in msg_section {
        match sect {
            Word(string) => {
                msg.push_str(string.as_str());
            }
            SpaceOrPunc(string) => {
                msg.push_str(string.as_str());
            }
        }
    }

    return msg;
}

fn split_into_word_vec(text: String) ->  Vec<Section> {
    let mut vec: Vec<Section> = Vec::new();

    let punc = r#" .,<>';:[]{}#~/!"£$%^&*()"#;
    let mut current_section: Section = Word(String::new());

    for char in text.chars() {
        if let Some(_)  = punc.find(char) {
            match &mut current_section {
                Word(_string) => {
                    vec.push(current_section);
                    current_section =  SpaceOrPunc(String::from(char));
                }
                SpaceOrPunc(string) => {
                    string.push(char);
                }
            }
        } else {
            match &mut current_section {
                Word(string) => {
                    string.push(char);
                }
                SpaceOrPunc(_string) => {
                    vec.push(current_section);
                    current_section = Word(String::from(char));
                }
            }
        }
    }
    vec.push(current_section);

    let first = vec.get(0).unwrap();
    if let Section::Word(string) = first {
        if string.len() == 0 {
            vec.remove(0);
        }
    }
    return vec;
}

pub fn initalise_rules() {
    let mut guard  = GLOBAL_RULES.lock().unwrap();
    let rules = guard.deref_mut();

    println!("{:?}",split_into_word_vec(String::from("This is a string with sentences. And here's a quote 'quote' ")));


    rules.push(Rule{ 
        name: String::from("Reversed"), 
        desc: String::from("Self explaintory"),
        weight: 1.0,
        process: |mut msg, _, _|  {
            let reversed_message = msg.text.chars().rev().collect::<String>();
            msg.text = reversed_message;
            return msg;
        }   
    });

    rules.push(Rule{ 
        name: String::from("Fck Vwls"), 
        desc: String::from("All vowels are removed"),
        weight: 1.0,
        process: |mut msg, _, _|  {
            let vowels = "aeiou".chars();
            let mut string: String= String::new();
            for letter in msg.text.chars() {
                for char in vowels.clone() {
                    if char == letter {break}
                }
                string.push(letter);
            }
            msg.text = string;
            return msg;
        }   
    });

    rules.push(Rule{ 
        name: String::from("Hopeless sacrastic"), 
        desc: String::from("All characters are randomly upper and lower case"),
        weight: 1.0,
        process: |mut msg, _, _|  {
            msg.text = msg.text.chars().map(|char| {
                if fastrand::bool() {
                    return char.to_uppercase().to_string().chars().last().unwrap();
                } else {
                    return char.to_lowercase().to_string().chars().last().unwrap();
                }
            }).collect();
            return msg;
        }   
    });

    rules.push(Rule{ 
        name: String::from("Palindrome"), 
        desc: String::from("Sentences are forced to be palindromic (same backwards and forwards ignoring punctation)"),
        weight: 1.0,
        process: |mut msg, _, _|  {
            let ignore = r#",.<>';:[]{}#~/!"£$%^&*()"#;
            let chars = msg.text.chars().filter(|char| {
                if let Some(_) = ignore.find(*char) {
                    return false;
                } else {
                    return true;
                };
            });
            let rev_chars = chars.clone().rev();
            if rev_chars.collect::<String>() == chars.collect::<String>() {
                return msg;
            } else {
                msg.text = msg.text + " test pand";
                return msg;
            }
        }   
    });


    rules.push(Rule{ 
        name: String::from("Even Steven"), 
        desc: String::from("Only sentences with an even amount of words are permited"),
        weight: 1.0,
        process: |msg, _, _|  {
            return msg;
        }   
    });
    rules.push(Rule{ 
        name: String::from("Repeating yourself"), 
        desc: String::from("If you do not use a word that has been sent in the last 30 messages, it is replaced by one that has"),
        weight: 1.0,
        process: |msg, _, _|  {
            return msg;
        }   
    });
    rules.push(Rule{ 
        name: String::from("Pretty much 1984"), 
        desc: String::from("You can only use words that appear in these restrictions. So, that does sucks. hope you like it"),
        weight: 32.0,
        process: |msg, _, _|  {
            return msg;
        }   
    });

    rules.push(Rule{ 
        name: String::from(".... .. ... / .-. ..- .-.. . / ... ..- -.-. -.- ..."), 
        desc: String::from("You can only use morse code characters of .-/ and spaces"),
        weight: 0.2,
        process: |msg, _, _|  {
            return msg;
        }   
    });

    rules.push(Rule{ 
        name: String::from("Gluttony"), 
        desc: String::from("You can only use words that include a single vowel or less"),
        weight: 0.2,
        process: |msg, _, _|  {
            return msg;
        }   
    });

    rules.push(Rule{ 
        name: String::from("The elephant in the room"), 
        desc: String::from("you must mention the elephant in the room"),
        weight: 0.2,
        process: |msg, _, _|  {
        
            return msg;


        }   
    });

    rules.push(Rule{ 
        name: String::from("Simplicity"), 
        desc: String::from("You must only use the top 250 most common english words"),
        weight: 0.2,
        process: |msg, _, _|  {
            return msg;
        }   
    });

    rules.push(Rule{ 
        name: String::from("Research document"), 
        desc: String::from("You cannot use personal pronouns"),
        weight: 0.2,
        process: |msg, _, _|  {
            return msg;
        }   
    });


    rules.push(Rule{ 
        name: String::from("English degree"), 
        desc: String::from("50% of your message must be valid english words above 10 letters"),
        weight: 0.2,
        process: |msg, _, _|  {
            return msg;
        }   
    });


    rules.push(Rule{ 
        name: String::from("Caveman speak"), 
        desc: String::from("All sentences are two words long and you used use me instead of I"),
        weight: 0.2,
        process: |msg, _, _|  {
            return msg;
        }   
    });

    rules.push(Rule{ 
        name: String::from("Ordered"), 
        desc: String::from("All words must be in alphabetical order in your message"),
        weight: 0.2,
        process: |msg, _, _|  {
            return msg;
        }   
    });

    rules.push(Rule{ 
        name: String::from("Unique"), 
        desc: String::from("You cannot use words used by the most 10 recent messages"),
        weight: 1.0,
        process: |msg, _, _|  {
            return msg;
        }   
    });


    rules.push(Rule{ 
        name: String::from("Corruption"), 
        desc: String::from("Random letters get swap with ones near themselves"),
        weight: 1.0,
        process: |mut msg, _, _|  {
            return msg;
        }   
    });


    


}