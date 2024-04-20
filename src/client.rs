use std::cell::RefCell;
use std::fs;
use std::fs::File;
use std::io::{Error, Write};
use std::net::{SocketAddr, TcpStream};
use std::path::PathBuf;
use std::rc::Rc;
use std::str::FromStr;

use display_info::DisplayInfo;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::display::*;
use crate::utils::config_dir;

#[derive(Serialize, Deserialize, Debug)]
pub struct Client {
    #[serde(skip)]
    pub ip: Option<SocketAddr>,
    #[serde(skip)]
    pub tcp: Option<Rc<RefCell<TcpStream>>>,
    #[serde(skip)]
    pub disp: Vec<Rc<RefCell<Display>>>,
    pub displays: Vec<Display>,
    pub cid: Uuid,
}

impl Client {
    pub fn new() -> Result<Rc<RefCell<Self>>, Error> {
        let client = Rc::new(RefCell::new(Client {
            ip: None,
            tcp: None,
            disp: Vec::new(),
            displays: DisplayInfo::all()
                .unwrap()
                .into_iter()
                .map(Display::from)
                .collect(),
            cid: Uuid::new_v4(),
        }));

        let mut client_mut = client.borrow_mut();

        // set displays
        client_mut.disp = DisplayInfo::all()
            .unwrap()
            .into_iter()
            .map(|disp| Rc::new(RefCell::new(Display::from(disp))))
            .collect::<Vec<Rc<RefCell<Display>>>>();

        // set client reference for displays
        for disp in client_mut.disp.iter_mut() {
            disp.borrow_mut().owner = Some(Rc::downgrade(&client));
        }

        // set cid from cid.txt
        let cid = client_mut.get_cid().unwrap();
        client_mut.cid = cid;

        Ok(client.clone())
    }

    pub fn connect(&mut self, server: &str) -> Result<(), Error> {
        let tcp = TcpStream::connect(server)?;
        self.tcp = Some(Rc::new(RefCell::new(tcp)));

        let encoded = bincode::serialize(&self).unwrap();
        let tcp = self.tcp.as_ref().unwrap();
        let mut tcp = tcp.borrow_mut();

        match tcp.write_all(&encoded.len().to_be_bytes()) {
            Ok(()) => {}
            Err(e) => return Err(e.into()),
        };

        match tcp.write_all(&encoded) {
            Ok(()) => {}
            Err(e) => return Err(e.into()),
        };

        Ok(())
    }

    pub fn listen(&self) -> Result<(), Error> {
        // TODO: implement from here

        Ok(())
    }

    fn get_cid(&self) -> Result<Uuid, Error> {
        let mut cid = Uuid::new_v4();

        // mkdir -p $path
        fs::create_dir_all(config_dir())?;

        let cid_file = config_dir().join("cid.txt");

        if cid_file.exists() {
            // read predefined cid from cid.txt
            let txt = fs::read_to_string(cid_file)?;
            cid = Uuid::from_str(&txt).unwrap();
        } else {
            // create new cid.txt
            let mut file = File::create(&cid_file)?;
            file.write_all(self.cid.to_string().as_bytes())?;
        }

        Ok(cid)
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
}
