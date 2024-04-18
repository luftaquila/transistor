use std::cell::RefCell;
use std::fs;
use std::io::{Error, ErrorKind, Read};
use std::net::TcpListener;
use std::rc::Rc;

use display_info::DisplayInfo;

use crate::client::*;
use crate::display::*;
use crate::utils::config_dir;

pub struct Server {
    tcp: TcpListener,
    clients: Vec<Rc<RefCell<Client>>>,
    displays: Vec<Rc<RefCell<Display>>>,
}

impl Server {
    pub fn new(port: u16) -> Result<Server, Error> {
        let server = Server {
            tcp: match TcpListener::bind(("0.0.0.0", port)) {
                Ok(tcp) => tcp,
                Err(e) => return Err(e.into()),
            },
            clients: Vec::new(),
            displays: Vec::new(),
        };

        // mkdir -p $path
        match fs::create_dir_all(config_dir()) {
            Ok(()) => {}
            Err(e) => return Err(e.into()),
        }

        Ok(server)
    }

    pub fn config(&mut self) -> Result<(), Error> {
        /* read config.json */
        let config = config_dir().join("config.json");

        if !config.exists() {
            return Err(Error::new(
                ErrorKind::NotFound,
                format!(
                    "no config.json found at {}",
                    config_dir().as_os_str().to_str().unwrap()
                ),
            ));
        }

        let json = match fs::read_to_string(&config) {
            Ok(json) => json,
            Err(e) => return Err(e.into()),
        };

        /* deserialize config.json */
        match serde_json::from_str(&json) {
            Ok(clients) => {
                let clients: Vec<Client> = clients;

                /* set clients */
                self.clients = clients
                    .into_iter()
                    .map(|c| Rc::new(RefCell::new(c)))
                    .collect();

                self.clients.iter_mut().for_each(|c| {
                    let mut c_ref = c.borrow_mut();

                    /* set disp for local ref, from deserialized displays */
                    c_ref.disp = c_ref
                        .displays
                        .clone()
                        .into_iter()
                        .map(|display| Rc::new(RefCell::new(display)))
                        .collect();

                    /* set owner and owner_type */
                    c_ref.disp.iter_mut().for_each(|d| {
                        d.borrow_mut().owner = Some(Rc::downgrade(c));
                        d.borrow_mut().owner_type = DisplayOwnerType::CLIENT;
                    });
                });
            }
            Err(e) => {
                return Err(Error::new(
                    ErrorKind::InvalidData,
                    format!("cannot parse config.json: {}", e.to_string()),
                ));
            }
        }

        /* analyze warpzones */
        if let Err(e) = self.analyze() {
            return Err(Error::new(
                ErrorKind::InvalidData,
                format!(
                    "{} is not valid: {}",
                    config.as_os_str().to_str().unwrap(),
                    e.to_string()
                ),
            ));
        }

        Ok(())
    }

    fn analyze(&mut self) -> Result<(), Error> {
        /* set system displays */
        let system_disp: Vec<Rc<RefCell<Display>>> = DisplayInfo::all()
            .unwrap()
            .into_iter()
            .map(|disp| Rc::new(RefCell::new(Display::from(disp))))
            .collect();

        /* set client displays */
        self.displays = self
            .clients
            .iter()
            .flat_map(|c| c.borrow().disp.clone())
            .collect();

        /* analyze warpzones for system displays ←→ client displays */
        for disp in system_disp.iter() {
            let mut disp_ref = disp.borrow_mut();

            disp_ref.owner = None;
            disp_ref.owner_type = DisplayOwnerType::SERVER;

            for target in self.displays.iter() {
                /* check overlap */
                if disp_ref.is_overlap(target) {
                    return Err(Error::new(
                        ErrorKind::InvalidInput,
                        format!(
                            "two displays are overlapping.\ndisp_A: {:#?}, disp_B: {:#?}",
                            disp_ref,
                            target.borrow()
                        ),
                    ));
                }

                /* create warpzones if touching each other */
                if let Some((start, end, direction)) = disp_ref.is_touch(target) {
                    disp_ref.warpzones.push(WarpZone {
                        start,
                        end,
                        direction,
                        to: Rc::downgrade(target),
                    });

                    target.borrow_mut().warpzones.push(WarpZone {
                        start,
                        end,
                        direction: direction.reverse(),
                        to: Rc::downgrade(disp),
                    });
                }
            }
        }

        /* analyze warpzones for client displays ←→ client displays */
        for (i, disp) in self.displays.iter().enumerate() {
            let mut disp_ref = disp.borrow_mut();

            for (j, target) in self.displays.iter().enumerate() {
                /* skip if the combination is already analyzed */
                if i >= j {
                    continue;
                }

                /* check overlap */
                if disp_ref.is_overlap(target) {
                    return Err(Error::new(
                        ErrorKind::InvalidInput,
                        format!(
                            "two displays are overlapping.\ndisp_A: {:#?}\ndisp_B: {:#?}",
                            disp_ref,
                            target.borrow()
                        ),
                    ));
                }

                /* create warpzones if touching each other */
                if let Some((start, end, direction)) = disp_ref.is_touch(target) {
                    disp_ref.warpzones.push(WarpZone {
                        start,
                        end,
                        direction,
                        to: Rc::downgrade(target),
                    });

                    target.borrow_mut().warpzones.push(WarpZone {
                        start,
                        end,
                        direction: direction.reverse(),
                        to: Rc::downgrade(disp),
                    });
                }
            }
        }

        /* check client displays without warpzone */
        for disp in self.displays.iter() {
            let disp = disp.borrow();

            if disp.warpzones.len() == 0 {
                return Err(Error::new(
                    ErrorKind::InvalidInput,
                    format!("isolated client display exists.\ndisp: {:#?}", disp),
                ));
            }
        }

        /* merge all configured displays */
        self.displays.extend(system_disp);

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
                        let mut client = client.borrow_mut();

                        if client.cid == incoming_client.cid {
                            /* verify configured displays */
                            for disp in client.displays.iter() {
                                for incoming_disp in incoming_client.displays.iter() {
                                    // TODO: check configured displays are same
                                }
                            }

                            client.tcp = Some(stream.try_clone().unwrap());
                            client.ip = Some(stream.peer_addr().unwrap());
                            verified[i] = true;

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
