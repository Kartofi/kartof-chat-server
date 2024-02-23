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
}
fn main() {
    let _ = listen("127.0.0.1:3012", |out| {
        let s: String = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(7)
            .map(char::from)
            .collect();
        move |msg: Message| {
            println!("{}", msg.to_string());
            let json: Payload = serde_json::from_str(&msg.to_string()).expect("msg");
            let data: Payload = Payload {
                from: s.clone().to_owned(),
                message: json.message.to_owned(),
            };
            let string_json: String = serde_json::to_string(&data).expect("msg");
            out.broadcast(string_json);
            Ok(())
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
