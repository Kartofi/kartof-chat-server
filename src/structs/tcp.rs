use rand::{distributions::Alphanumeric, Rng}; // 0.8

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use std::{
    ptr::null,
    time::{SystemTime, UNIX_EPOCH},
};

use ws::{CloseCode, Error, Handler, Message, Result, Sender};

static MAX_MESSAGE_SIZE: usize = 10 * 1024 * 1024;

#[derive(Clone, Serialize, Deserialize)]
struct Payload {
    from: String,
    message: String,
    file_data: String,
    file_type: String,
    time: u64,
}

pub struct Server {
    pub out: Sender,
    pub name: String,
    pub clients: Vec<String>,
}
impl Handler for Server {
    fn on_open(&mut self, shake: ws::Handshake) -> Result<()> {
        let name: String = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(7)
            .map(char::from)
            .collect();
        self.name = name.clone();
        self.out.send(self.name.clone()).unwrap();
        self.clients.push(name);
        Ok(())
    }
    fn on_close(&mut self, _: CloseCode, _: &str) {
        self.clients.retain(|x| x.to_string() != self.name);
    }
    fn on_error(&mut self, _: Error) {
        self.clients.retain(|x| x.to_string() != self.name);
    }
    fn on_message(&mut self, msg: Message) -> Result<()> {
        println!("Got Message from {}", self.name);
        if msg.to_string().len() > MAX_MESSAGE_SIZE {
            return Ok(());
        }
        let json: Value = serde_json::from_str(&msg.to_string()).expect("msg");
        if let Some(field) = json.get("request") {
            if field == "get_name" {
                self.out.send(self.name.clone()).unwrap();
            }
            return Ok(());
        } else {
            let payload: Payload = Payload {
                from: "".to_string(),
                message: json["message"].as_str().unwrap_or_default().to_string(),
                file_data: json["file_data"].as_str().unwrap_or_default().to_string(),
                file_type: json["file_type"].as_str().unwrap_or_default().to_string(),
                time: 0,
            };
            let start = SystemTime::now();
            let since_the_epoch: std::time::Duration = start
                .duration_since(UNIX_EPOCH)
                .expect("Time went backwards");
            let timestamp: u64 = since_the_epoch.as_secs();

            let data: Payload = Payload {
                from: self.name.to_owned(),
                message: payload.message.to_owned(),
                file_type: payload.file_type.to_owned(),
                file_data: payload.file_data.to_owned(),
                time: timestamp,
            };
            if payload.message.len() <= 0 && payload.file_data.len() <= 0 {
                return Ok(());
            }
            let string_json: String = serde_json::to_string(&data).expect("msg");

            self.out.broadcast(string_json).unwrap();
            Ok(())
        }
    }
}
