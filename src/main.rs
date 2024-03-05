use serde::{Deserialize, Serialize};

use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

use std::fs;

#[derive(Serialize, Deserialize)]
struct JsonData {
    name: u128,
    age: i64,
}

fn get_timestamp() -> u128 {
    let start = SystemTime::now();
    let since_the_epoch = start
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");

    let in_ms: u128 = since_the_epoch.as_millis();
    return in_ms;
}
const PORT: i64 = 3012;

use ws::listen;

mod structs;

fn main() {
    let users: Vec<String> = Vec::new();

    let mutex = Mutex::new(users);

    let arc = Arc::new(mutex);

    let _ = listen(format!("127.0.0.1:{PORT}"), |out| structs::tcp::Server {
        out: out,
        name: "".to_string(),
        clients: arc.clone(),
    });

    println!("Started listening on {}", PORT);
}
