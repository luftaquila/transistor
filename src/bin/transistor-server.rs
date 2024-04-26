use std::io::Error;

use transistor::*;

const PORT: u16 = 2426;

fn main() -> Result<(), Error> {
    println!("[INF] transistor server startup!");

    print_displays();

    let client_config = config_dir!().join("authorized_clients.json");
    let server = Server::new(PORT, client_config)?;

    println!("{:?}", server);

    Ok(())
}
