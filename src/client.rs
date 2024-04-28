use std::{fs, net::TcpStream};
use std::io::{Error, Read, Write};

use serde::{Deserialize, Serialize};

use crate::{config_dir, tcp_stream_write};

pub type Cid = u32;

#[derive(Serialize, Deserialize, Debug)]
pub struct AuthorizedClient {
    pub cid: Cid,
    // may have key-based auth in future
}

#[derive(Debug)]
pub struct Client {
    pub tcp: TcpStream,
    pub cid: Cid,
}

impl Client {
    pub fn new(server: &str) -> Result<Client, Error> {
        Ok(Client {
            tcp: TcpStream::connect(server)?,
            cid: load_or_generate_cid()?,
        })
    }

    pub fn start(&mut self) {
        tcp_stream_write!(self.tcp, self.cid);
    }
}

fn load_or_generate_cid() -> Result<Cid, Error> {
    let cid_file = config_dir!().join("cid");

    if cid_file.exists() {
        let txt = fs::read_to_string(cid_file)?;
        Ok(txt.parse().expect("[ERR] failed to load cid"))
    } else {
        let cid: Cid = rand::random();

        let mut file = fs::File::create(&cid_file)?;
        file.write_all(cid.to_string().as_bytes())?;

        Ok(cid)
    }
}
