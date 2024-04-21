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
    let mut client = client.borrow_mut();

    /* generate client.json */
    let path = client.to_json()?;

    println!("[INF] add generated client config to the server's config.json");
    println!("client.json: {}", path.as_os_str().to_str().unwrap());

    /* connect to server and transfer client info */
    client.connect(&args[1])?;

    client.listen()?;

    Ok(())
}
