use std::io::Error;

use transistor::*;

const PORT: u16 = 2426;

fn main() -> Result<(), Error> {
    println!("[INF] server startup!");

    let mut server = Server::new(PORT)?;

    server.init()?;
    server.start()?;

    // TODO: implement from here
    // capture();

    Ok(())
}

// fn init(mut stream: &TcpStream) -> Result<Vec<ClientDisplayInfo>, Error> {
//     /* deserialize client's display info */
//     let mut size = [0u8; 8];
//     stream.read_exact(&mut size)?;
//
//     let len = u64::from_be_bytes(size) as usize;
//     let mut buffer = vec![0u8; len];
//     stream.read_exact(&mut buffer[..len])?;
//
//     let displays: Vec<ClientDisplayInfo> =
//         bincode::deserialize(&buffer).map_err(|e| Error::new(ErrorKind::Other, e.to_string()))?;
//
//     println!(
//         "[INF] inbound client connection: {}",
//         stream.peer_addr().unwrap()
//     );
//
//     println!("[INF] client displays:");
//
//     for display in displays.iter() {
//         println!("  {:?}", display);
//     }
//
//     Ok(displays)
// }
//
// fn configure(client_displays: Vec<ClientDisplayInfo>) -> Result<(), Error> {
//     Ok(())
// }
//
// fn capture(mut stream: TcpStream) -> Result<(), Error> {
//     listen(move |event| {
//         println!("[EVT] {:?}", event);
//
//         /* TODO: find out which client to send event */
//
//         let encoded = bincode::serialize(&event).unwrap();
//         let size = encoded.len().to_be_bytes();
//
//         match stream.write_all(&encoded.len().to_be_bytes()) {
//             Ok(_) => (),
//             Err(e) => {
//                 eprintln!("[ERR] TCP stream write failed: {}", e);
//                 return;
//             }
//         }
//
//         match stream.write_all(&encoded) {
//             Ok(_) => (),
//             Err(e) => {
//                 eprintln!("[ERR] TCP stream write failed: {}", e);
//                 return;
//             }
//         }
//     })
//     .map_err(|e| Error::new(ErrorKind::Other, format!("{:?}", e)))?;
//
//     Ok(())
// }
