use rouille;
use rouille::router;


use serde::{Serialize, Deserialize};
use serde_json;

use std::time::{SystemTime, UNIX_EPOCH};

use std::fs;


#[derive(Serialize, Deserialize)]
struct JsonData {
    name: u128,
    age: i64
}

fn get_timestamp() -> u128 {
    let start = SystemTime::now();
    let since_the_epoch = start
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");

    let in_ms: u128 = since_the_epoch.as_millis();
    return in_ms;
}
const PORT:i64 = 8000;


use std::thread;
use std::net::{TcpListener, TcpStream, Shutdown};
use std::io::{Read, Write};


fn handle_client(mut stream: TcpStream) {
    let mut data = [0 as u8; 50]; // using 50 byte buffer
    while match stream.read(&mut data) {
        Ok(size) => {
            // echo everything!
            stream.write(&data[0..size]).unwrap();
            true
        },
        Err(_) => {
            println!("An error occurred, terminating connection with {}", stream.peer_addr().unwrap());
            stream.shutdown(Shutdown::Both).unwrap();
            false
        }
    } {}
}



fn main(){
    let listener = TcpListener::bind("0.0.0.0:3333").unwrap();
    // accept connections and process them, spawning a new thread for each one
    println!("Server listening on port 3333");
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                println!("New connection: {}", stream.peer_addr().unwrap());
                thread::spawn(move|| {
                    // connection succeeded
                    handle_client(stream)
                });
            }
            Err(e) => {
                println!("Error: {}", e);
                /* connection failed */
            }
        }
    }
    // close the socket server
    drop(listener);
    println!("Started listening on {}", PORT);
    rouille::start_server(format!("0.0.0.0:{}",PORT), move |request| {
        
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
