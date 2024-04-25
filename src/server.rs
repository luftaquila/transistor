use std::collections::HashMap;
use std::io::Error;
use std::net::TcpListener;

use display_info::DisplayInfo;
use mouce::Mouse;

use crate::display::*;
use crate::Client;

#[derive(Debug)]
pub struct Server {
    tcp: TcpListener,
    clients: HashMap<u32, Client>,
    displays: HashMap<u32, Display>,
    disp_list: AssignedDisplays,
    current: u32,
}

impl Server {
    pub fn new(port: u16) -> Result<Server, Error> {
        let display_vec: Vec<Display> = DisplayInfo::all()
            .unwrap()
            .into_iter()
            .map(Display::from)
            .collect();

        let system = display_vec.iter().map(|x| x.id).collect();
        let displays = display_vec.iter().map(|x| (x.id, x.clone())).collect();
        let current = display_vec.iter().find(|x| x.is_primary).unwrap().id;

        Ok(Server {
            tcp: TcpListener::bind(("0.0.0.0", port))?,
            clients: HashMap::new(),
            displays,
            disp_list: AssignedDisplays {
                system,
                client: Vec::new(),
            },
            current,
        })
    }
}
