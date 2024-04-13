use std::fs;
use std::fs::File;
use std::io::{Error, ErrorKind, Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::path::PathBuf;
use std::str::FromStr;

use directories::ProjectDirs;
use display_info::DisplayInfo;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/*********************************** utils ************************************/
#[macro_export]
macro_rules! config_dir {
    () => {
        ProjectDirs::from("io", "luftaquila", "transistor")
            .unwrap()
            .data_local_dir()
    };
}

pub fn print_displays() {
    println!("[INF] detected system displays:");
    let displays = DisplayInfo::all().unwrap();

    for display in displays {
        println!("  {:?}", display);
    }
}

/***************************** ClientDisplayInfo ******************************/
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ClientDisplayInfo {
    pub name: String,
    pub id: u32,
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
    pub rotation: f32,
    pub scale_factor: f32,
    pub frequency: f32,
    pub is_primary: bool,
}

impl From<DisplayInfo> for ClientDisplayInfo {
    fn from(item: DisplayInfo) -> Self {
        ClientDisplayInfo {
            name: item.name,
            id: item.id,
            // without raw_handle
            x: item.x,
            y: item.y,
            width: item.width,
            height: item.height,
            rotation: item.rotation,
            scale_factor: item.scale_factor,
            frequency: item.frequency,
            is_primary: item.is_primary,
        }
    }
}

/********************************** Client ************************************/
#[derive(Serialize, Deserialize, Debug)]
pub struct Client {
    #[serde(skip)]
    ip: Option<SocketAddr>,
    #[serde(skip)]
    tcp: Option<TcpStream>,
    displays: Vec<ClientDisplayInfo>,
    cid: Uuid,
}

impl Client {
    pub fn new() -> Result<Client, Error> {
        let mut client = Client {
            ip: None,
            tcp: None,
            displays: DisplayInfo::all()
                .unwrap()
                .into_iter()
                .map(ClientDisplayInfo::from)
                .collect(),
            cid: Uuid::new_v4(),
        };

        // mkdir -p $path
        match fs::create_dir_all(config_dir!()) {
            Ok(()) => {}
            Err(e) => return Err(e.into()),
        }

        // read predefined cid from cid.txt
        let config = config_dir!().join("cid.txt");

        if config.exists() {
            let txt = match fs::read_to_string(config) {
                Ok(txt) => txt,
                Err(e) => return Err(e.into()),
            };

            client.cid = Uuid::from_str(&txt).unwrap();
        } else {
            let path = config_dir!().join("cid.txt");

            let mut file = match File::create(&path) {
                Ok(file) => file,
                Err(e) => return Err(e.into()),
            };

            match file.write_all(client.cid.to_string().as_bytes()) {
                Ok(()) => {}
                Err(e) => return Err(e.into()),
            }
        }

        Ok(client)
    }

    pub fn to_json(&self) -> Result<PathBuf, Error> {
        let json = match serde_json::to_string_pretty(&self) {
            Ok(json) => json,
            Err(e) => return Err(e.into()),
        };

        let path = config_dir!().join("client.json");

        let mut file = match File::create(path.clone()) {
            Ok(file) => file,
            Err(e) => return Err(e.into()),
        };

        match file.write_all(json.as_bytes()) {
            Ok(file) => file,
            Err(e) => return Err(e.into()),
        };

        Ok(path)
    }

    pub fn connect(&mut self, server: &str) -> Result<(), Error> {
        self.tcp = match TcpStream::connect(server) {
            Ok(stream) => Some(stream),
            Err(e) => return Err(e.into()),
        };

        let encoded = bincode::serialize(&self).unwrap();

        match self
            .tcp
            .as_ref()
            .unwrap()
            .write_all(&encoded.len().to_be_bytes())
        {
            Ok(()) => {}
            Err(e) => return Err(e.into()),
        };

        match self.tcp.as_ref().unwrap().write_all(&encoded) {
            Ok(()) => {}
            Err(e) => return Err(e.into()),
        };

        Ok(())
    }

    pub fn listen(&self) -> Result<(), Error> {
        // TODO: implement from here

        Ok(())
    }
}

/********************************** Server ************************************/
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
        match fs::create_dir_all(config_dir!()) {
            Ok(()) => {}
            Err(e) => return Err(e.into()),
        }

        Ok(server)
    }

    pub fn config(&mut self) -> Result<(), Error> {
        /* read config.json */
        let config = config_dir!().join("config.json");

        if !config.exists() {
            return Err(Error::new(
                ErrorKind::NotFound,
                format!(
                    "no config.json found at {}",
                    config_dir!().as_os_str().to_str().unwrap()
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
