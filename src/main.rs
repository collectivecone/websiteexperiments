use std::{
    net::{TcpListener, TcpStream},
    collections::HashMap,
    thread::spawn,
};


mod utils;
mod experiments;

use utils::http::{
    Request,
    RequestType,
    HttpTypes,
    reply_to_get,
};

pub mod settings{
    use std::sync::RwLock;

    pub static GLOBAL_SETTINGS: RwLock<SettingsStruct> = RwLock::new(SettingsStruct{
        ignore_multiple_connections_per_ip: true,
    } );
    pub struct SettingsStruct {
        pub ignore_multiple_connections_per_ip: bool,
    }
}

fn main() {
    startup_experiments();
    perm_http_receiver();
}

fn startup_experiments() {
    spawn(|| crate::experiments::base::main());
}

fn perm_http_receiver() {
    let listener = TcpListener::bind("0.0.0.0:80").unwrap();

    for stream in listener.incoming() { 
        let mut stream = stream.unwrap();
        if let Some(request) = get_body_and_headers(&mut stream) {
            if request.headers.get("Upgrade").unwrap_or(&String::new()) == "websocket" {
                websocket_handling(stream,request);
            } else {
                website_handling(stream,request);
            }
        };
    }
}


fn websocket_handling(stream: TcpStream, request: Request ) {
    let link = request.request.request.clone();

    if link == "/" {
        crate::experiments::base::websocket_request(stream,request);
    }
}


fn website_handling(stream: TcpStream, request: Request) {

    let link = request.request.request.clone();

    println!("{}",link);

    if link == "/style.css" {
        reply_to_get(stream, "src/experiments/style.css");
    } else if link == "/favicon.ico" {
        reply_to_get(stream, "src/experiments/favicon.ico");
    } else if link == "/" {
        crate::experiments::base::http_request(stream,request);
    };
   // if link == "/" {
       
 //   }

    

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

