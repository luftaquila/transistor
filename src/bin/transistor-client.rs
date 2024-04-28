use std::env;
use std::io::{Error, ErrorKind::*};

use transistor::*;

fn main() -> Result<(), Error> {
    /* parse server adddress from command line arguments */
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        return Err(Error::new(
            InvalidInput,
            "[ERR] no server address specified",
        ));
    }

    let server = &args[1];

    println!("[INF] transistor client startup! server: {}", server);

    print_displays();

    let mut client = Client::new(server)?;

    client.start()?;

    loop {}

    Ok(())
}
