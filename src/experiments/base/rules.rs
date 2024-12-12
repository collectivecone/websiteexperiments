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
    pub time: u64,
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
use crate::utils::filter;

fn censore_word(msg: &mut String) {
    let original_len = msg.len();
    msg.clear();
    msg.push_str("*".repeat(original_len).as_str() );
}

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

fn split_into_word_vec(text: &String) ->  Vec<Section> {
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
            'outer: for letter in msg.text.chars() {
                for char in vowels.clone() {
                    if char == letter {continue 'outer}
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
            if rev_chars.collect::<String>().to_lowercase() == chars.collect::<String>().to_lowercase() {
                return msg;
            } else {
                msg.text = msg.text + " test pand";
                return msg;
            }
        }   
    });


    rules.push(Rule{ 
        name: String::from("Even Steven"), 
        desc: String::from("Only sentences with an even amount of words are permited or every other word is censored"),
        weight: 1.0,
        process: |mut msg, _, _|  {
            let mut sects = split_into_word_vec(&msg.text);
            let mut count: u32 = 0;
            for sect in &sects {
                if let Section::Word(_) = sect {
                    count += 1;
                }
            }
            if count % 2 == 0 {return msg} 
               
            let mut i = 0;
            for sect in sects.iter_mut() {
                if let Section::Word(string) = sect {
                    i += 1;
                    if i % 2 == 1 {
                        censore_word(string);
                    }
                }
            }
            msg.text = combine_section_vec_into_string(sects);

            return msg;
        }   
    });

    
    rules.push(Rule{ 
        name: String::from("Hive mind"), 
        desc: String::from("If you do not use a word that has been sent in the last 30 messages, it is replaced by one that has"),
        weight: 1.0,
        process: |mut msg: Message, _, msg_history|  {
            let mut word_list: Vec<String> = Vec::new();
            word_list.push(String::from("elephant"));
            for i in 0..(30.min(msg_history.len())) {
                let msg = msg_history.get(i).unwrap();
                let sects = split_into_word_vec(&msg.text);
                for sect in sects {
                    if let Word(string) = sect {
                        let a= string.to_lowercase();
                        word_list.push(a);
                    }
                }
            }
            println!("{:?}",word_list);

            let mut sects = split_into_word_vec(&msg.text);
            println!("{:?}",sects);
            for sect in &mut sects {
                match sect {
                    Word(string) => {
                        let mut exists: bool = false; 
                        for word in &word_list {
                            if word == string {
                                exists = true;
                                break
                            }
                        }
                        if !exists {
                            string.clear();
                            let random_word = &word_list[fastrand::usize(0..word_list.len())];
                            string.push_str(random_word.as_str());
                        }
                    },
                    _ => {},
                }
            }
            println!("{:?}",sects);

            msg.text = combine_section_vec_into_string(sects);
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
        name: String::from("Gluttony"), 
        desc: String::from("You can only use words that include a single vowel or its censored"),
        weight: 0.2,
        process: |mut msg, _, _|  {
            let vowels = "aeiou".chars();
            let mut sects = split_into_word_vec(&msg.text);
            for mut sect in &mut sects {
                if let Word(string) = &mut sect {
                    let mut count = 0;
                    for letter in string.chars() {
                        for char in vowels.clone() {
                            if char == letter {count += 1;}
                        }
                    }
                    if count != 1 {
                        censore_word(string);
                    }
                }
            }

            msg.text = combine_section_vec_into_string(sects);
            return msg;
        }   
    });

    rules.push(Rule{ 
        name: String::from("The elephant in the room"), 
        desc: String::from("you must mention the elephant in the room or your message just becomes 'Elephant'"),
        weight: 0.2,
        process: |mut msg, _, _|  {
            if let Some(_) = msg.text.to_lowercase().as_str().find("elephant") {
                return msg
            } else {
                msg.text = String::from("Elephant");
                return msg;
            };
        }   
    });

    rules.push(Rule{ 
        name: String::from("Simplicity"), 
        desc: String::from("You must only use the top 1000 most common english words or it is replaced by one that is"),
        weight: 0.2,
        process: |mut msg, _, _|  {
            let word_list: Vec<String> = filter::get_most_common_words();

            let mut sects = split_into_word_vec(&msg.text);
            for mut sect in &mut sects {
                match &mut sect {
                    Word(string) => {
                        let mut exists: bool = false; 
                        for word in &word_list {
                            if word.to_lowercase() == string.to_lowercase() {
                                exists = true;
                                break
                            }
                        }
                        if !exists {
                            string.clear();
                            let random_word = &word_list[fastrand::usize(0..word_list.len())];
                            string.push_str(random_word.as_str());
                        }
                    },
                    _ => {},
                }
            }
            msg.text = combine_section_vec_into_string(sects);
            return msg;
        }   
    });

    rules.push(Rule{ 
        name: String::from("Research document"), 
        desc: String::from("You cannot use personal pronouns; which are replaced by the name derek"),
        weight: 0.2,
        process: |mut msg, _, _|  {
            let word_list: Vec<&str> = vec!("I","me","you","he","she","they","them","him","her","hers","his","its","theirs","our","your");

            let mut sects = split_into_word_vec(&msg.text);
            for mut sect in &mut sects {
                match &mut sect {
                    Word(string) => {
                        let mut exists: bool = false; 
                        for word in &word_list {
                            if *word.to_lowercase() == string.as_str().to_lowercase() {
                                exists = true;
                                break
                            }
                        }
                        if exists {
                            string.clear();
                            string.push_str("derek");
                        }
                    },
                    _ => {},
                }
            }
            msg.text = combine_section_vec_into_string(sects);

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
        name: String::from("Ordered"), 
        desc: String::from("All words must be in alphabetical order in your message"),
        weight: 0.2,
        process: |mut msg, _, _|  {
            let mut sects = split_into_word_vec(&msg.text);
            let mut words: Vec<String> = Vec::new();
            for sect in &mut sects {
                if let Section::Word(string) = sect {
                    words.push(string.clone());
                } 
            };
            words.sort_by_key(|name| name.to_lowercase());
            let mut i: usize = 0;
            
            for sect in &mut sects {
                if let Section::Word(string) = sect {
                    string.clear();
                    string.push_str(words[i].as_str());
                    i += 1;
                }
            }

            msg.text = combine_section_vec_into_string(sects);
            return msg;
        }   
    });

    rules.push(Rule{ 
        name: String::from("Unique"), 
        desc: String::from("You cannot use words used by the most 10 recent messages"),
        weight: 1.0,
        process: |mut msg, _, msg_history|  {
            let mut word_list: Vec<String> = Vec::new();
            for i in 0..(30.min(msg_history.len() - 1)) {
                let msg = msg_history.get(i).unwrap();
                let sects = split_into_word_vec(&msg.text);
                for sect in sects {
                    if let Word(string) = sect {
                        let a= string.to_lowercase();
                        word_list.push(a);
                    }
                }
            }

            let mut sects = split_into_word_vec(&msg.text);
            for mut sect in &mut sects {
                match &mut sect {
                    Word(string) => {
                        let mut exists: bool = false; 
                        for word in &word_list {
                            if word.to_lowercase() == string.to_lowercase() {
                                exists = true;
                                break
                            }
                        }
                        if exists {
                            censore_word(string);
                        }
                    },
                    _ => {},
                }
            }

            msg.text = combine_section_vec_into_string(sects);


            return msg;
        }   
    });


    rules.push(Rule{ 
        name: String::from("Corruption"), 
        desc: String::from("Random letters get swap with ones near themselves"),
        weight: 1.0,
        process: |mut msg, _, _|  {
            let mut chars = msg.text.chars().collect::<Vec<_>>();

            for _ in 0..10 {
                let random_index = fastrand::usize(0..chars.len() - 1);

                let mut new_index = random_index;
                if fastrand::bool() {
                    new_index += 1
                } else {new_index -= 1}
                new_index = new_index.clamp(0,chars.len() - 1);

                chars.swap(random_index,new_index);
            }

            msg.text = chars.into_iter().collect();

            return msg;
        }   
    });
}