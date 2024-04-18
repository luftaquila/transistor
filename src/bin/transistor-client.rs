use std::env;
use std::io::{Error, ErrorKind};
use transistor::*;

fn main() -> Result<(), Error> {
    /* parse server adddress from command line arguments */
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        return Err(Error::new(
            ErrorKind::InvalidInput,
            "[ERR] no server address specified",
        ));
    }

    println!("[INF] client startup! server: {}", &args[1]);

    let client = Client::new()?;

    /* generate client.json */
    let path = client.borrow().to_json()?;

    println!("[INF] add generated client config to the server's config.json");
    println!("client.json: {}", path.as_os_str().to_str().unwrap());

    /* connect to server and transfer client info */
    client.borrow_mut().connect(&args[1])?;

    client.borrow().listen()?;

    loop {}

    Ok(())
}

// fn listen() {
//     let mut buffer = Vec::with_capacity(50);
//     let mut size = [0u8; 8];

//     loop {
//         match stream.read_exact(&mut size) {
//             Ok(_) => {}
//             Err(e) => {
//                 eprintln!("[ERR] event size read failed: {}", e);
//                 continue;
//             }
//         };

//         let len = u64::from_be_bytes(size) as usize;
//         buffer.resize(len, 0);

//         match stream.read_exact(&mut buffer[..len]) {
//             Ok(_) => {
//                 let event: Event = match bincode::deserialize(&buffer) {
//                     Ok(event) => event,
//                     Err(e) => {
//                         eprintln!("[ERR] event deserialization failed: {}", e);
//                         continue;
//                     }
//                 };

//                 println!("[EVT] <{}> {:?}", len, event);
//                 buffer.clear();
//             }
//             Err(e) => {
//                 eprintln!("[ERR] stream read failed: {}", e);
//                 break;
//             }
//         }
//     }
// }
