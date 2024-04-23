use std::{
    cell::{Cell, RefCell},
    fs,
    io::{Error, ErrorKind::*, Read, Write},
    mem,
    net::TcpListener,
    rc::{Rc, Weak},
};

use display_info::DisplayInfo;
use pixels::{Pixels, SurfaceTexture};
use rdev::*;
use winit::{
    dpi::{PhysicalPosition, PhysicalSize},
    event_loop::{ControlFlow, EventLoop},
    platform::run_return::EventLoopExtRunReturn,
    window::WindowBuilder,
};

use crate::{
    add_warpzone, client::*, client_point, create_warpgate, display::*, tcp_stream_read,
    tcp_stream_write, utils::config_dir, warp,
};

pub struct Server {
    tcp: TcpListener,
    clients: Vec<Rc<RefCell<Client>>>,
    displays: Vec<Rc<RefCell<Display>>>,
    current: Rc<RefCell<Option<Weak<RefCell<Display>>>>>,
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
            current: Rc::new(RefCell::new(None)),
        };

        // mkdir -p $path
        if let Err(e) = fs::create_dir_all(config_dir()) {
            return Err(e.into());
        }

        Ok(server)
    }

    pub fn config(&mut self) -> Result<(), Error> {
        /* read config.json */
        let config = config_dir().join("config.json");

        if !config.exists() {
            return Err(Error::new(
                NotFound,
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
                    InvalidData,
                    format!("cannot parse config.json: {}", e.to_string()),
                ));
            }
        }

        /* analyze warpzones */
        if let Err(e) = self.analyze() {
            return Err(Error::new(
                InvalidData,
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

        /* analyze warpzones for system displays */
        for (i, disp) in system_disp.iter().enumerate() {
            let mut disp_ref = disp.borrow_mut();

            disp_ref.owner = None;
            disp_ref.owner_type = DisplayOwnerType::SERVER;

            /* system displays ←→ client displays */
            for target in self.displays.iter() {
                /* check overlap */
                if disp_ref.is_overlap(target) {
                    return Err(Error::new(
                        InvalidInput,
                        format!(
                            "two displays are overlapping.\ndisp_A: {:#?}, disp_B: {:#?}",
                            disp_ref,
                            target.borrow()
                        ),
                    ));
                }
                /* create warpzones if touching each other */
                add_warpzone!(disp, disp_ref, target);
            }

            /* system displays ←→ system displays */
            for (j, target) in system_disp.iter().enumerate() {
                if i >= j {
                    continue;
                }
                /* no need to check overlap; just create warpzones */
                add_warpzone!(disp, disp_ref, target);
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
                        InvalidInput,
                        format!(
                            "two displays are overlapping.\ndisp_A: {:#?}\ndisp_B: {:#?}",
                            disp_ref,
                            target.borrow()
                        ),
                    ));
                }
                /* create warpzones if touching each other */
                add_warpzone!(disp, disp_ref, target);
            }
        }

        /* check client displays without warpzone */
        for disp in self.displays.iter() {
            let disp = disp.borrow();

            if disp.warpzones.len() == 0 {
                return Err(Error::new(
                    InvalidInput,
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

        let mut clients_verified: Vec<bool> = vec![false; self.clients.len()];

        // TODO: listen to clients in different thread for starting before all clients connected
        for stream in self.tcp.incoming() {
            match stream {
                Ok(mut stream) => {
                    /* receive client info; more enough bytes for bincode serialization overhead */
                    let mut buffer = [0u8; mem::size_of::<Client>() * 2];
                    tcp_stream_read!(stream, buffer);

                    /* deserialize transferred client info */
                    let incoming_client: Client = match bincode::deserialize(&buffer)
                        .map_err(|e| Error::new(InvalidData, e.to_string()))
                    {
                        Ok(client) => client,
                        Err(e) => return Err(e.into()),
                    };

                    /* verify client */
                    let mut incoming_verified = false;

                    for (i, client) in self.clients.iter_mut().enumerate() {
                        let mut client = client.borrow_mut();

                        if client.cid == incoming_client.cid {
                            /* verify configured displays */
                            let mut displays_verified: Vec<bool> =
                                vec![false; client.displays.len()];

                            for (j, disp) in client.displays.iter().enumerate() {
                                for incoming_disp in incoming_client.displays.iter() {
                                    if disp.name == incoming_disp.name
                                        && disp.id == incoming_disp.id
                                        && disp.width == incoming_disp.width
                                        && disp.height == incoming_disp.height
                                    {
                                        displays_verified[j] = true;
                                    }
                                }
                            }

                            let mut disp_verified = true;

                            for (j, item) in displays_verified.iter().enumerate() {
                                if *item == false {
                                    println!(
                                        "[WRN] client display confilcts in config and actual.\nclient: {}({})\ndisp: {:#?}",
                                        client.cid, stream.peer_addr().unwrap(), client.displays[j]
                                    );
                                    disp_verified = false;
                                    break;
                                }
                            }

                            if disp_verified == false {
                                break;
                            }

                            /* client and its displays are verified */
                            let tcp = Rc::new(RefCell::new(stream.try_clone().unwrap()));

                            /* TODO: tell warpzones to client */

                            /* set client network */
                            client.tcp = Some(tcp);
                            client.ip = Some(stream.peer_addr().unwrap());
                            clients_verified[i] = true;
                            incoming_verified = true;

                            println!(
                                "[INF] client {}({}) verified",
                                incoming_client.cid,
                                client.ip.unwrap()
                            );
                        }
                    }

                    if !incoming_verified {
                        println!(
                            "[WRN] verification failed for incoming client {}({})",
                            incoming_client.cid,
                            stream.peer_addr().unwrap()
                        );
                    }

                    /* check all clients verified */
                    let mut all_verified = true;

                    for item in clients_verified.iter() {
                        if *item == false {
                            all_verified = false;
                            break;
                        }
                    }

                    if all_verified == true {
                        break;
                    }
                }
                Err(e) => return Err(e.into()),
            }
        }

        println!("[INF] all clients connected and verified!");

        Ok(())
    }

    pub fn capture(self) -> Result<(), Error> {
        let current_disp = self.current.clone();

        let on_warp = Cell::new(false);

        grab(move |event| -> Option<rdev::Event> {
            /* skip event on rdev if cursor is warped */
            if on_warp.get() {
                return Some(event);
            }

            let mut current = current_disp.borrow_mut();

            match *current {
                /* if there is no current display, for the first time */
                None => {
                    match event.event_type {
                        EventType::MouseMove { x, y } => {
                            /* identify current display if mouse moves */
                            for disp in self.displays.iter() {
                                let d = disp.borrow();

                                /* skip client displays */
                                if let DisplayOwnerType::CLIENT = d.owner_type {
                                    continue;
                                }

                                /* set current display */
                                if x > d.x.into()
                                    && x < (d.x + d.width as i32).into()
                                    && y > d.y.into()
                                    && y < (d.y + d.height as i32).into()
                                {
                                    *current = Some(Rc::downgrade(disp));
                                    break;
                                }
                            }
                        }
                        _ => {} // else, just ignore
                    }
                }

                Some(ref cur) => {
                    let cur = cur.upgrade().unwrap();
                    let cur = cur.borrow();

                    let mut warp_point: Option<WarpPoint> = None;

                    match event.event_type {
                        EventType::MouseMove { x, y } => {
                            let x = x as i32;
                            let y = y as i32;

                            /* check if we are in warpzone */
                            for wz in cur.warpzones.iter() {
                                let margin = 1;

                                match wz.direction {
                                    ZoneDirection::HorizontalLeft => {
                                        if y >= wz.start - margin
                                            && y <= wz.end + margin
                                            && x <= cur.x + margin
                                        {
                                            drop(current);
                                            *current_disp.borrow_mut() = Some(wz.to.clone());
                                            warp_point = Some(client_point!(x, y, wz.to.clone()));
                                            break;
                                        }
                                    }
                                    ZoneDirection::HorizontalRight => {
                                        if y >= wz.start - margin
                                            && y <= wz.end + margin
                                            && x >= (cur.x + cur.width as i32) - margin
                                        {
                                            drop(current);
                                            *current_disp.borrow_mut() = Some(wz.to.clone());
                                            warp_point = Some(client_point!(x, y, wz.to.clone()));
                                            break;
                                        }
                                    }
                                    ZoneDirection::VerticalUp => {
                                        if x >= wz.start - margin
                                            && x <= wz.end + margin
                                            && y <= cur.y + margin
                                        {
                                            drop(current);
                                            *current_disp.borrow_mut() = Some(wz.to.clone());
                                            warp_point = Some(client_point!(x, y, wz.to.clone()));
                                            break;
                                        }
                                    }
                                    ZoneDirection::VerticalDown => {
                                        if x >= wz.start - margin
                                            && x <= wz.end + margin
                                            && y >= (cur.y + cur.height as i32) - margin
                                        {
                                            drop(current);
                                            *current_disp.borrow_mut() = Some(wz.to.clone());
                                            warp_point = Some(client_point!(x, y, wz.to.clone()));
                                            break;
                                        }
                                    }
                                }
                            }
                        }
                        _ => {}
                    };

                    /* no warp */
                    if warp_point.is_none() {
                        return Some(event);
                    }

                    /* update cur as current_disp changed */
                    let cur = current_disp.borrow().as_ref().unwrap().upgrade().unwrap();
                    let cur = cur.borrow();

                    /* check current display owner */
                    match cur.owner_type {
                        DisplayOwnerType::SERVER => {
                            return Some(event);
                        }

                        DisplayOwnerType::CLIENT => {
                            /* prepare client tcp stream */
                            let owner = cur.owner.as_ref().and_then(|o| o.upgrade()).unwrap();
                            let owner = owner.borrow_mut();
                            let tcp = owner.tcp.as_ref().unwrap();
                            let mut stream = tcp.borrow_mut();

                            /* tell client exact warp point */
                            println!("warp point: {:?}", warp_point);
                            tcp_stream_write!(stream, warp_point.unwrap());

                            /* warp to client; hide cursor at server */
                            on_warp.set(true);
                            // TODO: create warpgate earlier; to reduce delay
                            let (window, mut el) = create_warpgate!();
                            warp!(window, el);
                            on_warp.set(false);

                            /* ignore event in host system */
                            return None;
                        }
                    }
                }
            };

            Some(event)
        })
        .map_err(|e| Error::new(Other, format!("event capture error: {:?}", e)))?;

        Ok(())
    }
}
