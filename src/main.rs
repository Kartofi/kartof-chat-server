use rouille;
use rouille::router;

use serde::{Deserialize, Serialize};
use serde_json::json;

use std::time::{SystemTime, UNIX_EPOCH};

use std::fs;

use rand::{distributions::Alphanumeric, Rng}; // 0.8

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
const PORT: i64 = 8000;

use ws::{connect, CloseCode, Message, Result};

use ws::listen;

#[derive(Clone, Serialize, Deserialize)]
struct Payload {
    from: String,
    message: String,
    file_data: String,
    file_type: String,
    time: u64,
}

static MAX_MESSAGE_SIZE: usize = 10000000;

fn main() {
    let _ = listen("127.0.0.1:3012", |out| {
        let name: String = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(7)
            .map(char::from)
            .collect();
        out.send(name.clone()).unwrap();

        move |msg: Message| {
            println!("Got Message");
            if msg.to_string().len() > MAX_MESSAGE_SIZE {
                return Ok(());
            }

            if msg.clone().to_string() == "\"\"".to_string() {
                out.send(name.clone()).unwrap();
                return Ok(());
            } else {
                let json: Payload = serde_json::from_str(&msg.to_string()).expect("msg");

                let start = SystemTime::now();
                let since_the_epoch: std::time::Duration = start
                    .duration_since(UNIX_EPOCH)
                    .expect("Time went backwards");
                let timestamp: u64 = since_the_epoch.as_secs();

                let data: Payload = Payload {
                    from: name.to_owned(),
                    message: json.message.to_owned(),
                    file_type: json.file_type.to_owned(),
                    file_data: json.file_data.to_owned(),
                    time: timestamp,
                };
                if json.message.len() <= 0 {
                    return Ok(());
                }
                let string_json: String = serde_json::to_string(&data).expect("msg");

                out.broadcast(string_json).unwrap();
                Ok(())
            }
        }
    });
    println!("Started listening on {}", PORT);
    rouille::start_server(format!("0.0.0.0:{}", PORT), move |request| {
        router!(request,
            (GET) (/) => {
                match fs::read_to_string("./src/views/index.html") {
                    Ok(contents) => {
                        // If successful, print the contents
                        rouille::Response::html(contents)
                    }
                    Err(_) => {
                        // If there's an error, print the error
                        rouille::Response::text("ERROR!")
                    }
                }

            },
            (GET) (/api) => {


                let obj: JsonData = JsonData {name: get_timestamp(), age: 123};

                let json: String = serde_json::to_string(&obj).expect("REASON");

                rouille::Response::text(json)
            },
            _ => rouille::Response::empty_404()
        )
    });
}
