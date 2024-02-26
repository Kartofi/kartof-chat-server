use rouille;
use rouille::router;

use serde::{Deserialize, Serialize};
use serde_json::json;

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
const PORT: i64 = 8000;

use ws::listen;

mod structs;

fn main() {
    let _ = listen("127.0.0.1:3012", |out| structs::tcp::Server {
        out: out,
        name: "".to_string(),
        clients: vec![],
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
