use std::io::Error;

use transistor::*;

fn main() -> Result<(), Error> {
    println!("[INF] transistor server startup!");

    print_displays();

    let client_config = config_dir!("server").join("authorized_clients.json");
    let server = Server::new(1.0)?;

    server.start(client_config);

    Ok(())
}
