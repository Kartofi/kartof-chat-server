use rouille;
use rouille::router;

use std::fs;


fn main(){
    rouille::start_server("0.0.0.0:8000", move |request| {
        router!(request,
            (GET) (/) => {
                match fs::read_to_string("./src/views/index.html") {
                    Ok(contents) => {
                        // If successful, print the contents
                        rouille::Response::html(contents)
                    }
                    Err(err) => {
                        // If there's an error, print the error
                        rouille::Response::text("ERROR!")
                    }
                }
               
            },
            _ => rouille::Response::empty_404()
        )
    });
}
