use std::collections::HashMap;
use std::io::{Error, ErrorKind::*, Read, Write};
use std::{fs, net::TcpStream};
use std::{mem, u32};

use bincode::deserialize;
use display_info::DisplayInfo;
use serde::{Deserialize, Serialize};

use crate::display::*;
use crate::*;

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

    pub fn start(&mut self) -> Result<(), Error> {
        /* transmit cid to server */
        tcp_stream_write!(self.tcp, self.cid);

        /* receive display counts; 0 is unauthorized */
        let mut buffer = vec![0u8; mem::size_of::<u32>()];
        tcp_stream_read!(self.tcp, buffer);
        let disp_cnt: u32 = deserialize(&buffer).unwrap();

        if disp_cnt < 1 {
            return Err(Error::new(ConnectionRefused, "[ERR] authorization failed"));
        }

        /* receive server's current display configurations */
        tcp_stream_read_resize!(self.tcp, buffer);
        let server_disp_map: HashMap<Did, Display> = deserialize(&buffer).unwrap();
        let server_disp: Vec<Display> = server_disp_map.values().cloned().collect();

        /* configure our displays' attach position */
        let displays: Vec<Display> = set_display_position(server_disp);
        tcp_stream_write!(self.tcp, displays);

        /* wait server ack */
        tcp_stream_read!(self.tcp, buffer);

        if let HandshakeStatus::HandshakeErr = deserialize(&buffer).unwrap() {
            return Err(Error::new(ConnectionRefused, "[ERR] request rejected"));
        };

        Ok(())
    }
}

fn set_display_position(server_disp: Vec<Display>) -> Vec<Display> {
    let system_disp: Vec<Display> = DisplayInfo::all()
        .expect("[ERR] failed to get system displays")
        .into_iter()
        .map(|x| Display::from(x, 0)) // TODO: set CID
        .collect();

    // TODO
    system_disp
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
