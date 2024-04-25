use std::io::Error;

use transistor::*;

const PORT: u16 = 2426;

fn main() -> Result<(), Error> {
    println!("[INF] transistor server startup!");

    print_displays();

    let server = Server::new(PORT);

    println!("{:?}", server);

    Ok(())
}

