use std::collections::HashMap;
use std::fs;
use std::mem;
use std::io::{Error, ErrorKind::*, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::PathBuf;
use std::sync::{mpsc, Arc, Mutex};
use std::thread;

use display_info::DisplayInfo;
use mouce::Mouse;

use crate::{client::*, tcp_stream_read};
use crate::display::*;

#[derive(Debug)]
pub struct Server {
    clients: Arc<Mutex<HashMap<Cid, Client>>>,
    displays: HashMap<Did, Display>,
    disp_ids: AssignedDisplays,
    current: Did,
}

impl Server {
    pub fn new() -> Result<Server, Error> {
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
            clients: Arc::new(Mutex::new(HashMap::new())),
            displays,
            disp_ids: AssignedDisplays {
                system,
                client: Vec::new(),
            },
            current,
        })
    }

    pub fn start(&self, port: u16, client_config: PathBuf) {
        // let (tx, rx) = mpsc::channel();
        let clients = self.clients.clone();

        thread::spawn(move || {
            handle_client(port, clients, client_config);
        });
    }
}

fn handle_client(port: u16, clients: Arc<Mutex<HashMap<Cid, Client>>>, config: PathBuf) -> Result<(), Error> {
    let tcp = TcpListener::bind(("0.0.0.0", port)).expect("[ERR] TCP binding failed");
    let authorized = authorized_clients(config).expect("[ERR] failed to read client config");

    for mut stream in tcp.incoming().filter_map(Result::ok) {
        /* read cid from remote client */
        let mut buffer = [0u8; mem::size_of::<Cid>()];
        tcp_stream_read!(stream, buffer);
        let cid = Cid::from_be_bytes(buffer);

        /* reject not known client */
        if !authorized.contains(&cid) {

        }

        let client = Client {
            tcp: stream,
            cid,
        };

        clients.lock().unwrap().insert(cid, client);

        /* send current displays */

        /* receive attach request */
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
