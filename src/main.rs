use std::{
    fs,
    io::{prelude::*, BufReader},
    net::{TcpListener, TcpStream},
    collections::HashMap,
};


mod utils;
mod experiments;

use utils::http::{Request,RequestType,HttpTypes};

fn main() {
    let listener = TcpListener::bind("127.0.0.1:7879").unwrap();

    for stream in listener.incoming() { 
        let mut stream = stream.unwrap();
        println!("new");
        if let Some(request) = get_body_and_headers(&mut stream) {
            if request.headers.get("Upgrade").unwrap_or(&String::new()) == "websocket" {
                println!("?");
                websocket_handling(stream,request);
            } else {
                website_handling(stream,request);
            }
        };
    }
}



fn websocket_handling(stream: TcpStream, request: Request ) {
   println!("{:?}",request);

    let link = request.request.request.clone();

    if link == "/" {
        crate::experiments::base::websocket_request(stream,request);
    }
}


fn website_handling(stream: TcpStream, request: Request) {
    println!("{:?}",request);

    let link = request.request.request.clone();

    if link == "/" {
        crate::experiments::base::http_request(stream,request);
    }
}

pub fn get_body_and_headers(stream: &mut TcpStream) -> Option<Request> { 
    let mut buf = [0; 10000];
    if let Ok(len) = stream.peek(&mut buf) {
      let mut buf = vec![0;len];
      let _ = stream.peek(&mut buf).unwrap();
   
      if let Ok(whole_request) = String::from_utf8(buf.to_vec()) {
        let mut header_str: String = String::new();
        let mut header = HashMap::new();
        let mut request: RequestType = RequestType{http_type : HttpTypes::Get, request: String::new()} ;
    
        let mut lines = whole_request.lines();
     
        loop {
           let line = lines.next();
           match line {
              Some(line) => {
              if line.len() < 3 {
                 break;
              }
           
              header_str.push_str( line);
              header_str.push('\n');
           },
              None => return None,
           }
        }
     
     
        for (i, line) in header_str.lines().into_iter() .enumerate() {
           let thing: Vec<&str> = line.split(" ").collect();
           if i == 0 {
              match thing.get(0) {
                 Some(t) =>  {
                       match t {
                          &"GET" => {request.http_type = HttpTypes::Get}
                          &"POST" => {request.http_type = HttpTypes::Post}
     
     
                          _ => {return None},
                       }
                 },
                 None => {return None}
              }
     
              match thing.get(1) {
                 Some(t) => {request.request = t.to_string() },
                 None => {return None},
              }
           }
           else {
              let mut x = thing.get(0).unwrap_or(&"").to_string();
              let y = thing.get(1).unwrap_or(&"").to_string();
              if !x.is_empty() && !y.is_empty() {
                 x.pop();
     
                 header.insert(x,y);
              }
           }
        }
     
        let mut body = String::new();
        for line in lines {
            body.push_str(line);
        }
        
        return Some(Request{request: request, body: body, headers: header})
      };
    } 
    return None
}