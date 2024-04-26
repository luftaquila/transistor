use std::collections::HashMap;
use std::fs;
use std::io::{Error, ErrorKind::*};
use std::net::TcpListener;
use std::path::PathBuf;

use display_info::DisplayInfo;
use mouce::Mouse;

use crate::client::*;
use crate::display::*;

#[derive(Debug)]
pub struct Server {
    tcp: TcpListener,
    clients: HashMap<Cid, Client>,
    authorized: Vec<Cid>,
    displays: HashMap<Did, Display>,
    disp_ids: AssignedDisplays,
    current: Did,
}

impl Server {
    pub fn new(port: u16, client_config: PathBuf) -> Result<Server, Error> {
        let disp: Vec<Display> = DisplayInfo::all()
            .expect("[ERR] failed to get system displays")
            .into_iter()
            .map(Display::from)
            .collect();

        if disp.len() == 0 {
            return Err(Error::new(NotFound, "[ERR] system display not found"));
        }

        let system = disp.iter().map(|x| x.id).collect();
        let current = disp.iter().find(|x| x.is_primary).unwrap_or(&disp[0]).id;
        let displays = disp.into_iter().map(|x| (x.id, x)).collect();

        Ok(Server {
            tcp: TcpListener::bind(("0.0.0.0", port))?,
            clients: HashMap::new(),
            authorized: authorized_clients_from_config(client_config)
                .expect("[ERR] failed to read client config"),
            displays,
            disp_ids: AssignedDisplays {
                system,
                client: Vec::new(),
            },
            current,
        })
    }
}

fn authorized_clients_from_config(file: PathBuf) -> Result<Vec<Cid>, Error> {
    if !file.exists() {
        return Err(Error::new(
            NotFound,
            format!("{} not found", file.as_os_str().to_str().unwrap()),
        ));
    }

    let json = fs::read_to_string(&file)?;
    let clients: Vec<AuthorizedClient> = serde_json::from_str(&json)?;
    let clients_cid = clients.iter().map(|x| x.cid).collect();

    Ok(clients_cid)
}
