use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub enum Request {
    Write { key: String, value: String },
    Read { key: String },
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Response {
    Ok,
    Value(String),
    NotFound,
    Error(String),
}