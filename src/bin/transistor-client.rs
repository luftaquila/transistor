use std::env;
use std::io::{Error, ErrorKind::*};

use transistor::*;

const PORT: u16 = 2426;

fn main() -> Result<(), Error> {
    /* parse server adddress from command line arguments */
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        return Err(Error::new(
            InvalidInput,
            "[ERR] no server address specified",
        ));
    }

    println!("[INF] transistor client startup! server: {}", &args[1]);

    print_displays();

    Ok(())
}
