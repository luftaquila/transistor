use std::io::Error;

use transistor::*;

fn main() -> Result<(), Error> {
    println!("[INF] transistor server startup!");

    print_displays();

    let client_config = config_dir!("server").join("authorized_clients.json");
    let server = Server::new()?;

    server.start(client_config);

    println!("{:?}", server);

    loop {}

    Ok(())
}
