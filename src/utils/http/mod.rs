use std::{
    collections::HashMap,
};

#[derive(Debug, PartialEq, Eq)]
pub enum HttpTypes {
   Post,
   Get
}
#[derive(Debug)]
pub struct RequestType {
   pub http_type: HttpTypes,
   pub request: String,
}

#[derive(Debug)]
pub struct Request {
    pub request: RequestType,
    pub body: String,
    pub headers: HashMap<String,String>,
}
