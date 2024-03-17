use rdev::*;
use std::env;
use std::io::Read;
use std::net::TcpStream;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("[ERR]: no server address specified");
        return;
    }

    let server = &args[1];

    let mut stream = match TcpStream::connect(server) {
        Ok(stream) => stream,
        Err(e) => {
            eprintln!("[ERR] server {} connection failed: {}", server, e);
            return;
        }
    };

    println!("[INF] server connected!");

    let mut buffer = Vec::with_capacity(50);
    let mut size = [0u8; 8];

    loop {
        match stream.read_exact(&mut size) {
            Ok(_) => {},
            Err(e) => {
                eprintln!("[ERR] event size read failed: {}", e);
                continue;
            }
        };

        let len = u64::from_be_bytes(size) as usize;
        buffer.resize(len, 0);

        match stream.read_exact(&mut buffer[..len]) {
            Ok(_) => {
                let event: Event = match bincode::deserialize(&buffer) {
                    Ok(event) => event,
                    Err(e) => {
                        eprintln!("[ERR] event deserialization failed: {}", e);
                        continue;
                    }
                };

                println!("[EVT] <{}> {:?}", len, event);
                buffer.clear();
            }
            Err(e) => {
                eprintln!("[ERR] stream read failed: {}", e);
                break;
            }
        }
    }
}

