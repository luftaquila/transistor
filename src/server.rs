use std::collections::HashMap;
use std::fs;
use std::io::{Error, ErrorKind::*};
use std::mem;
use std::net::TcpListener;
use std::path::PathBuf;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{mpsc, Arc, RwLock};
use std::thread;

use bincode::deserialize;
use display_info::DisplayInfo;
use mouce::common::MouseEvent;
use mouce::Mouse;

use crate::client::*;
use crate::display::*;
use crate::*;

#[derive(Debug)]
enum Action {
    Warp,
    Move,
}

#[derive(Debug)]
struct Message {
    disp: Did,
    action: Action,
    x: i32,
    y: i32,
}

#[derive(Debug)]
pub struct Server {
    clients: Arc<RwLock<HashMap<Cid, Client>>>,
    displays: Arc<RwLock<HashMap<Did, Display>>>,
    disp_ids: Arc<RwLock<AssignedDisplays>>,
    current: Arc<RwLock<Did>>,
}

impl Server {
    pub fn new() -> Result<Server, Error> {
        // mkdir -p
        fs::create_dir_all(config_dir!("server"))?;

        let mut disp: Vec<Display> = DisplayInfo::all()
            .expect("[ERR] failed to get system displays")
            .into_iter()
            .map(|x| Display::from(x, SERVER_CID))
            .collect();

        if disp.len() == 0 {
            return Err(Error::new(NotFound, "[ERR] system display not found"));
        }

        let system = disp.iter().map(|x| x.id).collect();
        let current = Arc::new(RwLock::new(
            disp.iter().find(|x| x.is_primary).unwrap_or(&disp[0]).id,
        ));
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
            disp_ids: Arc::new(RwLock::new(AssignedDisplays {
                system,
                client: Vec::new(),
            })),
            current,
        })
    }

    pub fn start(&self, authorized: PathBuf) {
        /* message exchange channels */
        let (tx, rx1) = mpsc::channel::<Message>();
        let (tx1, rx) = mpsc::channel::<Message>();

        let clients = self.clients.clone();
        let displays = self.displays.clone();
        let disp_ids = self.disp_ids.clone();

        /* spawn tcp handler thread */
        thread::spawn(move || {
            handle_client(clients, displays, disp_ids, authorized, tx1, rx1);
        });

        let mut mouce = Mouse::new();

        /* find out the current display */
        let disp_ids = self.disp_ids.clone();
        let displays = self.displays.clone();
        let (x, y) = mouce
            .get_position()
            .expect("[ERR] cannot get cursor position");

        {
            let display_map = displays.read().unwrap();

            for disp in disp_ids.read().unwrap().system.iter() {
                let d = display_map.get(disp).unwrap();

                if x > d.x && x < (d.x + d.width) && y > d.y && y < (d.y + d.height) {
                    *self.current.write().unwrap() = d.id;
                    break;
                }
            }
        }

        /* listen mouse events */
        let current = self.current.clone();

        let hook = mouce.hook(Box::new(move |e| {
            let (x, y) = if let MouseEvent::AbsoluteMove(x, y) = e {
                (*x, *y)
            } else {
                return;
            };

            /* check if we are in warpzone */
            let mut cur_id = current.write().unwrap();
            let disps = displays.read().unwrap();
            let cur = disps.get(&*cur_id).unwrap();

            let mut warp_point: Option<(i32, i32)> = None;

            for wz in cur.warpzones.iter() {
                match wz.direction {
                    ZoneDirection::HorizontalLeft => {
                        if y >= wz.start - MARGIN && y <= wz.end + MARGIN && x <= cur.x + MARGIN {
                            *cur_id = wz.to;
                            let to = disps.get(&*cur_id).unwrap();
                            warp_point = Some((x - to.x, y - to.y));
                            break;
                        }
                    }
                    ZoneDirection::HorizontalRight => {
                        if y >= wz.start - MARGIN
                            && y <= wz.end + MARGIN
                            && x >= (cur.x + cur.width as i32) - MARGIN
                        {
                            *cur_id = wz.to;
                            let to = disps.get(&*cur_id).unwrap();
                            warp_point = Some((x - to.x, y - to.y));
                            break;
                        }
                    }
                    ZoneDirection::VerticalUp => {
                        if x >= wz.start - MARGIN && x <= wz.end + MARGIN && y <= cur.y + MARGIN {
                            *cur_id = wz.to;
                            let to = disps.get(&*cur_id).unwrap();
                            warp_point = Some((x - to.x, y - to.y));
                            break;
                        }
                    }
                    ZoneDirection::VerticalDown => {
                        if x >= wz.start - MARGIN
                            && x <= wz.end + MARGIN
                            && y >= (cur.y + cur.height as i32) - MARGIN
                        {
                            *cur_id = wz.to;
                            let to = disps.get(&*cur_id).unwrap();
                            warp_point = Some((x - to.x, y - to.y));
                            break;
                        }
                    }
                }
            }

            // no go
            if warp_point.is_none() {
                return;
            }

            /* warp sequence begin */
            let (x, y) = warp_point.unwrap();

            // transmit warp point
            if let Err(e) = tx.send(Message {
                disp: *cur_id,
                action: Action::Warp,
                x,
                y,
            }) {
                eprintln!("[ERR] mpsc tx failed: {}", e);
            }

            // warp
            // TODO: winit
        }));

        if let Err(e) = hook {
            eprintln!("[ERR] event hook failed: {}", e);
        }
    }
}

fn handle_client(
    clients: Arc<RwLock<HashMap<Cid, Client>>>,
    mut displays: Arc<RwLock<HashMap<Did, Display>>>,
    disp_ids: Arc<RwLock<AssignedDisplays>>,
    authorized: PathBuf,
    tx: Sender<Message>,
    rx: Receiver<Message>,
) {
    /* spawn transceiver thread */
    let client_list = clients.clone();

    thread::spawn(move || {
        transceive(client_list, tx, rx);
    });

    /* get authorized client list */
    let authorized =
        get_authorized_clients(authorized).expect("[ERR] failed to read client config");

    let tcp = TcpListener::bind(("0.0.0.0", PORT)).expect("[ERR] TCP binding failed");

    /* start handshaking with client */
    for mut stream in tcp.incoming().filter_map(Result::ok) {
        let ip = stream.peer_addr().unwrap();

        /* read cid from remote client */
        let mut buffer = vec![0u8; mem::size_of::<Cid>()];

        if let Err(e) = tcp_read(&mut stream, &mut buffer) {
            eprintln!("[ERR] client {} handshake failed: {}", ip, e);
            continue;
        };

        let cid = deserialize(&buffer).unwrap();

        // reject unknown client
        if !authorized.contains(&cid) {
            if let Err(e) = tcp_write(&mut stream, 0) {
                eprintln!("[ERR] client {} handshake failed: {}", ip, e);
                continue;
            };
        }

        // transmit display counts to client
        if let Err(e) = tcp_write(&mut stream, displays.read().unwrap().len() as u32) {
            eprintln!("[ERR] client {} handshake failed: {}", ip, e);
            continue;
        };

        // transmit current displays
        {
            let disp = displays.read().unwrap();

            if let Err(e) = tcp_write(&mut stream, disp.clone()) {
                eprintln!("[ERR] client {} handshake failed: {}", ip, e);
                continue;
            };
        }

        // receive display attach request
        if let Err(e) = tcp_read(&mut stream, &mut buffer) {
            eprintln!("[ERR] client {} handshake failed: {}", ip, e);
            continue;
        };

        let mut client_disp: Vec<Display> = deserialize(&buffer).unwrap();

        // update warpzones for new displays
        let new = match create_warpzones_hashmap(&mut displays, &mut client_disp) {
            Ok(new) => new,
            Err(e) => {
                eprintln!("[ERR] invalid request from client {} : {}", ip, e);
                continue;
            }
        };

        // transmit ack
        if let Err(e) = tcp_write(&mut stream, HandshakeStatus::HandshakeOk as i32) {
            eprintln!("[ERR] client {} handshake failed: {}", ip, e);
            continue;
        };

        // add accepted client and display list
        let client = Client {
            tcp: stream,
            cid,
            displays: Vec::new(), // not using at server
        };

        clients.write().unwrap().insert(cid, client);
        disp_ids.write().unwrap().client.extend(new);

        println!("[INF] client {} connected!", ip);
    }
}

fn transceive(
    clients: Arc<RwLock<HashMap<Cid, Client>>>,
    tx: Sender<Message>,
    rx: Receiver<Message>,
) {
    match rx.recv() {
        Ok(msg) => {
            println!("msg: {:?}", msg);
        }
        Err(e) => {
            eprintln!("[ERR] message receive failed: {}", e);
        }
    }
}

fn get_authorized_clients(file: PathBuf) -> Result<Vec<Cid>, Error> {
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
