use std::fs;
use std::fs::File;
use std::io::{Error, Write};
use std::net::{SocketAddr, TcpStream};
use std::path::PathBuf;
use std::str::FromStr;

use display_info::DisplayInfo;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::utils::config_dir;

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

#[derive(Serialize, Deserialize, Debug)]
pub struct Client {
    #[serde(skip)]
    pub ip: Option<SocketAddr>,
    #[serde(skip)]
    tcp: Option<TcpStream>,
    displays: Vec<ClientDisplayInfo>,
    pub cid: Uuid,
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
        match fs::create_dir_all(config_dir()) {
            Ok(()) => {}
            Err(e) => return Err(e.into()),
        }

        // read predefined cid from cid.txt
        let config = config_dir().join("cid.txt");

        if config.exists() {
            let txt = match fs::read_to_string(config) {
                Ok(txt) => txt,
                Err(e) => return Err(e.into()),
            };

            client.cid = Uuid::from_str(&txt).unwrap();
        } else {
            let mut file = match File::create(&config) {
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

        let path = config_dir().join("client.json");

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
