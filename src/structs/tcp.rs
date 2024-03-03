use rand::{distributions::Alphanumeric, Rng}; // 0.8

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::structs::data_processing::{compress, decompress};

use std::default;
use std::ptr::null;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};
use ws::{CloseCode, Error, Handler, Message, Result, Sender};

static MAX_MESSAGE_SIZE: usize = 10 * 1024 * 1024;

#[derive(Clone, Serialize, Deserialize)]
struct ClientMessage {
    from: String,
    message: String,
    file_data: String,
    file_type: String,
    time: u64,
}
#[derive(Clone, Serialize, Deserialize)]
struct Request {
    request: RequestTypes,
    name: Option<String>,
    users: Option<Vec<String>>,
}
pub struct Server {
    pub out: Sender,
    pub name: String,
    pub clients: Arc<Mutex<Vec<String>>>,
}
#[derive(Clone, Serialize, Deserialize, PartialEq, Eq)]
enum RequestTypes {
    GetName = 1,
}
impl Handler for Server {
    fn on_open(&mut self, _shake: ws::Handshake) -> Result<()> {
        let name: String = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(7)
            .map(char::from)
            .collect();
        self.name = name.clone();

        let mut clients = self.clients.lock().unwrap();
        clients.push(name);

        let string_json: String = serde_json::to_string(&Request {
            request: RequestTypes::GetName,
            name: Some(self.name.clone()),
            users: Some(clients.clone()),
        })
        .expect("msg");

        self.out.broadcast(string_json).unwrap();
        Ok(())
    }
    fn on_close(&mut self, _: CloseCode, _: &str) {
        let mut clients = self.clients.lock().unwrap();
        clients.retain(|x| x.to_string() != self.name);
        let string_json: String = serde_json::to_string(&Request {
            request: RequestTypes::GetName,
            name: Some(self.name.clone()),
            users: Some(clients.clone()),
        })
        .expect("msg");

        self.out.broadcast(string_json).unwrap();
    }
    fn on_error(&mut self, _: Error) {
        let mut clients = self.clients.lock().unwrap();
        clients.retain(|x| x.to_string() != self.name);
        let string_json: String = serde_json::to_string(&Request {
            request: RequestTypes::GetName,
            name: Some(self.name.clone()),
            users: Some(clients.clone()),
        })
        .expect("msg");

        self.out.broadcast(string_json).unwrap();
    }
    fn on_message(&mut self, msg: Message) -> Result<()> {
        println!("Got Message from {}", self.name);
        if msg.to_string().len() > MAX_MESSAGE_SIZE {
            println!("Message is too large");
            return Ok(());
        }
        let compressed_bytes: Vec<u8> = match base64::decode(&msg.clone().to_string()) {
            Ok(bytes) => bytes,
            Err(e) => {
                eprintln!("Base64 decoding error: {}", e);
                Vec::new() // Exit the function if decoding fails
            }
        };
        if compressed_bytes.len() == 0 {
            println!("Invalid compressed data!");
            return Ok(());
        }
        let decompressed_result: std::prelude::v1::Result<String, std::io::Error> =
            decompress(&compressed_bytes);
        let decompressed: String = match decompressed_result {
            Ok(data) => data,
            Err(error) => String::default(),
        };

        if decompressed == String::default() {
            println!("Invalid compressed data!");
            return Ok(());
        }
        let json_result = serde_json::from_str(&decompressed);

        let json: Value = match json_result {
            Ok(data) => data,
            Err(error) => Value::default(),
        };
        if json == Value::default() {
            println!("Invalid json!");
            return Ok(());
        }
        if let Some(field) = json.get("request") {
            let req: Request = serde_json::from_str(&decompressed).expect("msg");

            if req.request == RequestTypes::GetName {
                let clients = self.clients.lock().unwrap();
                let string_json: String = serde_json::to_string(&Request {
                    request: RequestTypes::GetName,
                    name: Some(self.name.clone()),
                    users: Some(clients.clone()),
                })
                .expect("msg");
                let compressed: String = compress(&string_json).unwrap();
                self.out.send(compressed).unwrap();
            }
            return Ok(());
        } else {
            let start = SystemTime::now();
            let since_the_epoch: std::time::Duration = start
                .duration_since(UNIX_EPOCH)
                .expect("Time went backwards");
            let timestamp: u64 = since_the_epoch.as_secs();

            let payload: ClientMessage = ClientMessage {
                from: self.name.to_owned(),
                message: json["message"].as_str().unwrap_or_default().to_string(),
                file_data: json["file_data"].as_str().unwrap_or_default().to_string(),
                file_type: json["file_type"].as_str().unwrap_or_default().to_string(),
                time: timestamp,
            };

            if payload.message.len() <= 0 && payload.file_data.len() <= 0 {
                return Ok(());
            }
            let string_json: String = serde_json::to_string(&payload).expect("msg");
            let compressed: String = compress(&string_json).unwrap();
            self.out.broadcast(compressed).unwrap();
            Ok(())
        }
    }
}
