use tungstenite::{connect, Message};
use url::Url;

fn main() {
    // URL of the WebSocket server
    let server_url = Url::parse("ws://localhost:8001").expect("Invalid URL");

    // Connect to the WebSocket server
    let (mut socket, response) =
        connect(server_url).expect("Cannot connect to the scheduling system");

    println!("Connected to the server");
    println!("Response HTTP code: {}", response.status());
    println!("Response contains the following headers:");
    for (ref header, _value) in response.headers() {
        println!("* {}", header);
    }

    // Send a message to the server
    socket
        .write_message(Message::Text("Hello WebSocket".into()))
        .expect("Failed to send a message");

    // Close the connection
    socket.close(None).expect("Failed to close the connection");
}
