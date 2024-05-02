use std::collections::HashMap;
use std::fs;
use std::io::{stdout, Error, ErrorKind::*, Write};
use std::mem;
use std::net::TcpStream;

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
    pub displays: Vec<Display>,
}

impl Client {
    pub fn new(server: &str) -> Result<Client, Error> {
        // mkdir -p
        fs::create_dir_all(config_dir!("client"))?;

        let cid = load_or_generate_cid()?;

        Ok(Client {
            tcp: TcpStream::connect(server)?,
            cid,
            displays: DisplayInfo::all()
                .expect("[ERR] failed to get system displays")
                .into_iter()
                .map(|x| Display::from(x, cid))
                .collect(),
        })
    }

    pub fn start(&mut self) -> Result<(), Error> {
        // transmit cid to server
        if let Err(e) = tcp_write(&mut self.tcp, self.cid) {
            return Err(Error::new(
                ConnectionRefused,
                format!("handshake failed: {:?}", e),
            ));
        };

        /* receive display counts; 0 is unauthorized */
        let mut buffer = vec![0u8; mem::size_of::<u32>()];

        if let Err(e) = tcp_read(&mut self.tcp, &mut buffer) {
            return Err(Error::new(
                ConnectionRefused,
                format!("handshake failed: {:?}", e),
            ));
        };

        let disp_cnt: u32 = deserialize(&buffer).unwrap();

        if disp_cnt < 1 {
            return Err(Error::new(ConnectionRefused, "authorization failed"));
        }

        // receive server's current display configurations
        if let Err(e) = tcp_read(&mut self.tcp, &mut buffer) {
            return Err(Error::new(
                ConnectionRefused,
                format!("handshake failed: {:?}", e),
            ));
        };

        let server_disp_map: HashMap<Did, Display> = deserialize(&buffer).unwrap();
        let server_disp: Vec<Display> = server_disp_map.values().cloned().collect();

        /* configure our displays' attach position and transmit to server */
        self.set_display_position(server_disp);

        if let Err(e) = tcp_write(&mut self.tcp, self.displays.clone()) {
            return Err(Error::new(
                ConnectionRefused,
                format!("handshake failed: {:?}", e),
            ));
        };

        /* wait server ack */
        if let Err(e) = tcp_read(&mut self.tcp, &mut buffer) {
            return Err(Error::new(
                ConnectionRefused,
                format!("handshake failed: {:?}", e),
            ));
        };

        if let HandshakeStatus::HandshakeErr = deserialize(&buffer).unwrap() {
            return Err(Error::new(ConnectionRefused, "[ERR] request rejected"));
        };

        println!("[INF] connected!");

        Ok(())
    }

    fn set_display_position(&mut self, server_conf: Vec<Display>) {
        let displays = &mut self.displays;

        let file = config_dir!("client").join("client_config.json");

        if file.exists() {
            let json = match fs::read_to_string(file) {
                Ok(json) => json,
                Err(_) => {
                    eprint!("[WRN] invalid client_config.json");
                    return prompt_display_position(displays, server_conf);
                }
            };

            let config: Vec<Display> = match serde_json::from_str(&json) {
                Ok(vec) => vec,
                Err(_) => {
                    eprint!("[WRN] invalid client_config.json");
                    return prompt_display_position(displays, server_conf);
                }
            };

            /* check all displays in config are correct */
            let mut cnt = 0;

            for disp in displays.iter() {
                for conf in config.iter() {
                    // the only param that could identify displays
                    if disp.name == conf.name {
                        cnt += 1;
                        continue;
                    }
                }
            }

            if cnt != displays.len() {
                eprint!("[WRN] client_config.json does not match with current system displays");
                return prompt_display_position(displays, server_conf);
            }

            /* set positions with config; misconfigurations will be checked in the server */
            for disp in displays.iter_mut() {
                for conf in config.iter() {
                    if disp.name == conf.name {
                        disp.x = conf.x;
                        disp.y = conf.y;
                        continue;
                    }
                }
            }
        } else {
            // config not exists
            return prompt_display_position(displays, server_conf);
        }
    }
}

fn load_or_generate_cid() -> Result<Cid, Error> {
    let cid_file = config_dir!("client").join("cid.txt");

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

fn prompt_display_position(displays: &mut Vec<Display>, server_conf: Vec<Display>) {
    println!("########## display setup ##########");
    println!("[INF] current server displays:");

    for (i, d) in server_conf.iter().enumerate() {
        println!(
            "  [{:2}] x: {:4}, y: {:4}, width: {:4}, height: {:4}",
            i, d.x, d.y, d.width, d.height
        );
    }

    println!("[INF] please enter the attach position of the each display");
    println!("[INF] (x, y) is the coordinate of the upper left corner of the display");

    let mut i = 0;
    let tot = displays.len();

    while i < tot {
        let d = displays.get_mut(i).unwrap();
        println!(
            "[{:2}/{}] {} - width: {:4}, height: {:4}",
            i, tot, d.name, d.width, d.height
        );

        print!("  x coordinate: ");
        stdout().flush().unwrap();

        match stdin_i32() {
            Ok(x) => d.x = x,
            Err(_) => {
                eprintln!("  [ERR] invalid input");
                continue;
            }
        }

        print!("  y coordinate: ");
        stdout().flush().unwrap();

        match stdin_i32() {
            Ok(y) => d.y = y,
            Err(_) => {
                eprintln!("  [ERR] invalid input");
                continue;
            }
        }

        // confirm input
        loop {
            print!(
                "[CONFIRM] {} - x: {}, y: {}, width: {:4}, height: {:4} [y/n/p]: ",
                d.name, d.x, d.y, d.width, d.height
            );
            stdout().flush().unwrap();

            let ch = match stdin_char() {
                Ok(ch) => ch,
                Err(_) => {
                    continue;
                }
            };

            match ch {
                'y' => {
                    i += 1; // next display
                    break;
                }
                'n' => {
                    break; // current display again
                }
                'p' => {
                    if i > 0 {
                        i -= 1; // previous display
                    }
                    break;
                }
                _ => {
                    continue;
                }
            }
        }
    }

    loop {
        print!("[INF] save current config? [y/n]: ");
        stdout().flush().unwrap();

        let ch = match stdin_char() {
            Ok(ch) => ch,
            Err(_) => {
                continue;
            }
        };

        match ch {
            'y' => {
                let path = config_dir!("client").join("client_config.json");
                let path_str = path.as_os_str().to_str().unwrap();

                let mut file = match fs::File::create(&path) {
                    Ok(file) => file,
                    Err(e) => {
                        eprintln!("[ERR] failed to create {}: {}", path_str, e);
                        break;
                    }
                };

                match serde_json::to_string_pretty(&displays) {
                    Ok(json) => {
                        if let Err(e) = file.write_all(json.as_bytes()) {
                            eprintln!("[ERR] failed to write to {}: {}", path_str, e);
                        }

                        println!("[INF] config saved at {}", path_str);
                    }
                    Err(e) => eprintln!("[ERR] failed to save config into file: {}", e),
                }
                break;
            }
            'n' => {
                break;
            }
            _ => {
                continue;
            }
        }
    }
}
