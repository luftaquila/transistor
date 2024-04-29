use std::collections::HashMap;
use std::fs;
use std::io::{Error, ErrorKind::*, Read, Write};
use std::mem;
use std::net::TcpListener;
use std::path::PathBuf;
use std::sync::{mpsc, Arc, RwLock};
use std::thread;

use bincode::deserialize;
use display_info::DisplayInfo;
use mouce::Mouse;

use crate::client::*;
use crate::display::*;
use crate::*;

#[derive(Debug)]
pub struct Server {
    clients: Arc<RwLock<HashMap<Cid, Client>>>,
    displays: Arc<RwLock<HashMap<Did, Display>>>,
    disp_ids: AssignedDisplays,
    current: Did,
}

impl Server {
    pub fn new() -> Result<Server, Error> {
        let mut disp: Vec<Display> = DisplayInfo::all()
            .expect("[ERR] failed to get system displays")
            .into_iter()
            .map(|x| Display::from(x, SERVER_CID))
            .collect();

        if disp.len() == 0 {
            return Err(Error::new(NotFound, "[ERR] system display not found"));
        }

        let system = disp.iter().map(|x| x.id).collect();
        let current = disp.iter().find(|x| x.is_primary).unwrap_or(&disp[0]).id;
        let displays = Arc::new(RwLock::new(
            disp.iter().map(|x| (x.id, x.clone())).collect(),
        ));

        let mut dummy = disp.clone();

        /* create warpzone twice with reverse order to write correctly in disp, not dummy */
        if let Err(_) = create_warpzones(&mut disp, &mut dummy, true) {
            return Err(Error::new(InvalidData, "[ERR] system display init failed"));
        };

        if let Err(_) = create_warpzones(&mut dummy, &mut disp, true) {
            return Err(Error::new(InvalidData, "[ERR] system display init failed"));
        };

        Ok(Server {
            clients: Arc::new(RwLock::new(HashMap::new())),
            displays,
            disp_ids: AssignedDisplays {
                system,
                client: Vec::new(),
            },
            current,
        })
    }

    pub fn start(&self, client_config: PathBuf) {
        // let (tx, rx) = mpsc::channel();
        let clients = self.clients.clone();
        let displays = self.displays.clone();

        thread::spawn(move || {
            handle_client(clients, displays, client_config);
        });
    }
}

fn handle_client(
    clients: Arc<RwLock<HashMap<Cid, Client>>>,
    displays: Arc<RwLock<HashMap<Did, Display>>>,
    config: PathBuf,
) -> Result<(), Error> {
    let tcp = TcpListener::bind(("0.0.0.0", PORT)).expect("[ERR] TCP binding failed");
    let authorized = authorized_clients(config).expect("[ERR] failed to read client config");

    for mut stream in tcp.incoming().filter_map(Result::ok) {
        /* read cid from remote client */
        let mut buffer = vec![0u8; mem::size_of::<Cid>()];
        tcp_stream_read!(stream, buffer);
        let cid = deserialize(&buffer).unwrap();

        /* reject not known client */
        if !authorized.contains(&cid) {
            tcp_stream_write!(stream, 0);
        }

        /* transmit display counts to client */
        tcp_stream_write!(stream, displays.read().unwrap().len() as u32);

        /* transmit current displays */
        let disp = displays.read().unwrap();
        tcp_stream_write!(stream, *disp);

        /* receive display attach request */
        tcp_stream_read_resize!(stream, buffer);
        let client_disp: Vec<Display> = deserialize(&buffer).unwrap();

        /* calculate warpzones for new displays */
        // TODO

        /* transmit ack */
        tcp_stream_write!(stream, HandshakeStatus::HandshakeOk);

        /* add accepted client */
        // TODO
        let client = Client { tcp: stream, cid };
        clients.write().unwrap().insert(cid, client);
    }

    Ok(())
}

fn authorized_clients(file: PathBuf) -> Result<Vec<Cid>, Error> {
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
