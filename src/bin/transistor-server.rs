use std::io::Error;

use transistor::*;

const PORT: u16 = 2426;

fn main() -> Result<(), Error> {
    println!("[INF] server startup!");

    let mut server = Server::new(PORT)?;

    /* read config.json */
    server.config()?;

    /* wait for clients connection */
    server.start()?;

    /* start monitoring inputs */
    server.capture()?;

    Ok(())
}
