use std::{mem, u32};
use std::{fs, net::TcpStream};
use std::io::{Error, ErrorKind::*, Read, Write};

use serde::{Deserialize, Serialize};

use crate::{config_dir, tcp_stream_read, tcp_stream_write};

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

    pub fn start(&mut self) -> Result<(), Error>{
        /* send cid to server */
        tcp_stream_write!(self.tcp, self.cid);

        /* get display counts; 0 is unauthorized */
        let mut buffer = [0u8; mem::size_of::<u32>()];
        tcp_stream_read!(self.tcp, buffer);
        let disp_cnt: u32 = bincode::deserialize(&buffer).unwrap();

        println!("cnt: {}", disp_cnt);

        if disp_cnt < 1 {
            return Err(Error::new(NotConnected, "[ERR] authorization failed"));
        }

        Ok(())
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
