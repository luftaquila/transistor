use std::fs;
use std::fs::File;
use std::io::{Error, ErrorKind, Write};
use std::net::{SocketAddr, TcpListener};
use std::path::PathBuf;
use std::str::FromStr;

use directories::ProjectDirs;
use display_info::DisplayInfo;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

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

pub fn print_displays() {
    println!("[INF] detected system displays:");
    let displays = DisplayInfo::all().unwrap();

    for display in displays {
        println!("  {:?}", display);
    }
}

#[macro_export]
macro_rules! config_dir {
    () => {
        ProjectDirs::from("io", "luftaquila", "transistor")
            .unwrap()
            .data_local_dir()
    };
}

fn empty_addr() -> SocketAddr {
    "0.0.0.0:0000".parse().unwrap()
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Client {
    #[serde(default = "empty_addr")]
    ip: SocketAddr,
    displays: Vec<ClientDisplayInfo>,
    cid: Uuid,
}

impl Client {
    pub fn new() -> Result<Client, Error> {
        let mut client = Client {
            ip: empty_addr(),
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
            Err(e) => {
                return Err(Error::new(
                    e.kind(),
                    format!(
                        "cannot access config dir {}: {}",
                        config_dir!().as_os_str().to_str().unwrap(),
                        e.to_string()
                    ),
                ));
            }
        }

        // read predefined cid from cid.txt
        let config = config_dir!().join("cid.txt");

        if config.exists() {
            let txt = fs::read_to_string(config)?;
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
        // TODO: error handlings
        let json = serde_json::to_string_pretty(&self)?;
        let path = config_dir!().join("client.json");
        let mut file = File::create(path.clone())?;
        file.write_all(json.as_bytes())?;

        Ok(path)
    }
}

pub struct Server {
    tcp: TcpListener,
    clients: Vec<Client>,
}

impl Server {
    pub fn new(port: u16) -> Result<Server, Error> {
        let server = Server {
            tcp: TcpListener::bind(("0.0.0.0", port)).expect(&format!("port {} bind failed", port)),
            clients: Vec::new(),
        };

        // mkdir -p $path
        match fs::create_dir_all(config_dir!()) {
            Ok(()) => {}
            Err(e) => {
                return Err(Error::new(
                    e.kind(),
                    format!(
                        "cannot access config dir {}: {}",
                        config_dir!().as_os_str().to_str().unwrap(),
                        e.to_string()
                    ),
                ));
            }
        }

        Ok(server)
    }

    pub fn init(&mut self) -> Result<(), Error> {
        // read config.json
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

        let json = fs::read_to_string(config)?;

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
        println!("[INF] waiting for clients...");

        for stream in self.tcp.incoming() {
            match stream {
                Ok(stream) => {
                    println!("{:?}", stream.peer_addr());
                    // TODO: validate client info with config.json
                }
                Err(e) => eprintln!("[ERR] TCP connection failed: {}", e),
            }
        }

        Ok(())
    }
}
