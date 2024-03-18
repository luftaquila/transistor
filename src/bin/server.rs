use rdev::*;
use std::io::Write;
use std::net::TcpListener;
use display_info::DisplayInfo;

const PORT: u16 = 2426;

fn main() {
    let _display = DisplayInfo::all().unwrap();

    let listener = TcpListener::bind(("0.0.0.0", PORT))
        .expect(&format!("[ERR] port {} bind failed!", PORT));

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                println!("[INF] inbound connection: {}", stream.peer_addr().unwrap());

                if let Err(error) = listen(move |event| {
                    println!("[EVT] {:?}", event);

                    let encoded = bincode::serialize(&event).unwrap();
                    let size = encoded.len().to_be_bytes();

                    if let Err(e) = stream.write_all(&size) {
                        eprintln!("[ERR] TCP stream write failed: {}", e);
                    } else if let Err(e) = stream.write_all(&encoded) {
                        eprintln!("[ERR] TCP stream write failed: {}", e);
                    }
                }) {
                    eprintln!("[ERR] input capture error: {:?}", error);
                }
            }
            Err(e) => {
                eprintln!("[ERR] TCP connection failed: {}", e);
            }
        }
    }
}
