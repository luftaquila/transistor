use std::io::{Error, ErrorKind, Read, Write};
use std::net::{TcpListener, TcpStream};

use display_info::DisplayInfo;
use rdev::*;
use transistor::client_displayinfo::ClientDisplayInfo;

const PORT: u16 = 2426;

fn main() -> Result<(), Error> {
    println!("[INF] server startup!");

    println!("[INF] detected system displays:");
    let displays = DisplayInfo::all().unwrap();

    for display in displays {
        println!("  {:?}", display);
    }

    let listener =
        TcpListener::bind(("0.0.0.0", PORT)).expect(&format!("[ERR] port {} bind failed!", PORT));

    println!("[INF] waiting for clients...");

    for stream in listener.incoming() {
        if let Err(e) = stream {
            eprintln!("[ERR] TCP connection failed: {}", e);
            continue;
        }

        let client_displays = init(stream.unwrap())?;
    }

    Ok(())
}

fn init(mut stream: TcpStream) -> Result<Vec<ClientDisplayInfo>, Error> {
    /* deserialize client's display info */
    let mut size = [0u8; 8];
    stream.read_exact(&mut size)?;

    let len = u64::from_be_bytes(size) as usize;
    let mut buffer = vec![0u8; len];
    stream.read_exact(&mut buffer[..len])?;

    let displays: Vec<ClientDisplayInfo> =
        bincode::deserialize(&buffer).map_err(|e| Error::new(ErrorKind::Other, e.to_string()))?;

    println!(
        "[INF] inbound client connection: {}",
        stream.peer_addr().unwrap()
    );

    println!("[INF] client displays:");

    for display in displays.iter() {
        println!("  {:?}", display);
    }

    Ok(displays)
}

// println!("[INF] inbound connection: {}", stream.peer_addr().unwrap());

// if let Err(error) = listen(move |event| {
//     println!("[EVT] {:?}", event);

//     let encoded = bincode::serialize(&event).unwrap();
//     let size = encoded.len().to_be_bytes();

//     if let Err(e) = stream.write_all(&size) {
//         eprintln!("[ERR] TCP stream write failed: {}", e);
//     } else if let Err(e) = stream.write_all(&encoded) {
//         eprintln!("[ERR] TCP stream write failed: {}", e);
//     }
// }) {
//     eprintln!("[ERR] input capture error: {:?}", error);
// }
