use std::fs;
use std::io::{Error, ErrorKind, Read};
use std::net::TcpListener;

use crate::client::Client;
use crate::utils::config_dir;

pub struct Server {
    tcp: TcpListener,
    clients: Vec<Client>,
}

impl Server {
    pub fn new(port: u16) -> Result<Server, Error> {
        let server = Server {
            tcp: match TcpListener::bind(("0.0.0.0", port)) {
                Ok(tcp) => tcp,
                Err(e) => return Err(e.into()),
            },
            clients: Vec::new(),
        };

        // mkdir -p $path
        match fs::create_dir_all(config_dir()) {
            Ok(()) => {}
            Err(e) => return Err(e.into()),
        }

        Ok(server)
    }

    pub fn config(&mut self) -> Result<(), Error> {
        /* read config.json */
        let config = config_dir().join("config.json");

        if !config.exists() {
            return Err(Error::new(
                ErrorKind::NotFound,
                format!(
                    "no config.json found at {}",
                    config_dir().as_os_str().to_str().unwrap()
                ),
            ));
        }

        let json = match fs::read_to_string(config) {
            Ok(json) => json,
            Err(e) => return Err(e.into()),
        };

        match serde_json::from_str(&json) {
            Ok(clients) => {
                self.clients = clients;
            }
            Err(e) => {
                return Err(Error::new(
                    ErrorKind::InvalidData,
                    format!("cannot parse config.json: {}", e.to_string()),
                ));
            }
        }

        Ok(())
    }

    pub fn start(&mut self) -> Result<(), Error> {
        println!(
            "[INF] waiting for {} configured clients to be connected...",
            self.clients.len()
        );

        let mut verified: Vec<bool> = vec![false; self.clients.len()];

        for stream in self.tcp.incoming() {
            match stream {
                Ok(mut stream) => {
                    /* receive client info */
                    let mut size = [0u8; 8];
                    match stream.read_exact(&mut size) {
                        Ok(()) => {}
                        Err(e) => return Err(e.into()),
                    }

                    let len = u64::from_be_bytes(size) as usize;
                    let mut buffer = vec![0u8; len];
                    match stream.read_exact(&mut buffer[..len]) {
                        Ok(()) => {}
                        Err(e) => return Err(e.into()),
                    }

                    /* deserialize transferred client info */
                    let incoming_client: Client = match bincode::deserialize(&buffer)
                        .map_err(|e| Error::new(ErrorKind::InvalidData, e.to_string()))
                    {
                        Ok(client) => client,
                        Err(e) => return Err(e.into()),
                    };

                    /* verify client */
                    for (i, client) in self.clients.iter_mut().enumerate() {
                        if client.cid == incoming_client.cid {
                            verified[i] = true;
                            client.ip = Some(stream.peer_addr().unwrap());

                            println!(
                                "client {}({}) verified",
                                incoming_client.cid,
                                client.ip.unwrap()
                            );
                        }
                    }

                    /* check all clients verified */
                    let mut ok = true;

                    for i in 0..verified.len() {
                        if verified[i] == false {
                            ok = false;
                            break;
                        }
                    }

                    /* all configured clients connected and verified */
                    if ok == true {
                        break;
                    }
                }
                Err(e) => return Err(e.into()),
            }
        }

        println!("[INF] all clients connected and verified!");

        Ok(())
    }

    pub fn capture(&self) -> Result<(), Error> {
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

        Ok(())
    }
}
