use std::thread;
use std::time;

use websocket::client::ClientBuilder;
use websocket::{Message, OwnedMessage};


const CONNECTION: &'static str = "wss://stream.binance.com:9443/ws";
const targets: [&str; 2] = ["btcusdt", "ethusdt"]; // more then 1 in stream couses sequence bug
const update_time_ms: i32 = 1000; // 1000 ms or 100 ms
const depth: i32 = 5; // Valid are 5, 10, or 20.
const life_time_sec: u64 = 10;
fn main() {
    let mut client = ClientBuilder::new(CONNECTION)
        .unwrap()
        // .connect_insecure()
        .connect(None)
        .unwrap();
	println!("Successfully connected");
    {
        let mut message: String = r#"{"method": "SUBSCRIBE","params":["#.to_owned();
        for i in 0..targets.len() {
            message.push_str("\"");
            message.push_str(targets[i]);
            message.push_str("@depth");
            message.push_str(&depth.to_string());
            message.push_str("@");
            message.push_str(&update_time_ms.to_string());
            message.push_str("ms\"");
            if i != (targets.len() - 1) {
                message.push_str(",");
            }
        }
        message.push_str(r#"],"id":1}"#);
        client.send_message(&OwnedMessage::Text(message.to_string()));
    }
    {
        let message = client.recv_message();
    }
    let receive_loop = thread::spawn(move || {
        let mut targetsIt: i32 = 0;
        let targetsItPtr = &mut targetsIt;
        loop {
            let message = client.recv_message();
            let message = match message {
                Ok(m) => m,
                Err(e) => {
                    println!("Receive Loop: {:?}", e);
                    return;
                }
            };
            match message {
                OwnedMessage::Close(_) => {
                    client.send_message(&OwnedMessage::Close(None));
                    return;
                }
                OwnedMessage::Ping(data) => {
                    client.send_message(&OwnedMessage::Pong(data));
                }
                OwnedMessage::Text(value) => {
                    let parsed = json::parse(&value).unwrap();
                    if !(parsed["bids"][0][0].is_null() && parsed["asks"][0][0].is_null()) {
                        *targetsItPtr = (*targetsItPtr + 1) % (targets.len() as i32);
                        println!("[{:?}] OB -> Binance : {} {} x {}", chrono::offset::Utc::now(), targets[*targetsItPtr as usize], parsed["bids"][0][0], parsed["asks"][0][0]);
                    } else {
                        println!("[{:?}] OB -> Binance : error parse : {:?}", chrono::offset::Utc::now(), value);
                    }
                }
                _ => {
                    println!("[{:?}] OB -> Binance : {:?}", chrono::offset::Utc::now(), message);
                }
            }
        }
    });
    let tsleep = time::Duration::from_millis(life_time_sec * 1000);
    thread::sleep(tsleep);
}