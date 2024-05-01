use std::io::{stdin, Error, ErrorKind::*, Read, Write};
use std::net::TcpStream;

use display_info::DisplayInfo;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub enum HandshakeStatus {
    HandshakeOk,
    HandshakeErr,
}

pub fn print_displays() {
    println!("[INF] detected system displays:");

    for display in DisplayInfo::all().unwrap() {
        println!("  {:?}", display);
    }

    println!();
}

pub fn stdin_i32() -> Result<i32, Error> {
    let mut input = String::new();

    if let Err(_) = stdin().read_line(&mut input) {
        return Err(Error::new(InvalidInput, "stdin read failure"));
    };

    match input.trim().parse() {
        Ok(i) => Ok(i),
        Err(_) => {
            return Err(Error::new(InvalidInput, "invalid input"));
        }
    }
}

pub fn stdin_char() -> Result<char, Error> {
    let mut input = String::new();

    if let Err(_) = stdin().read_line(&mut input) {
        return Err(Error::new(InvalidInput, "stdin read failure"));
    };

    match input.trim().parse() {
        Ok(i) => Ok(i),
        Err(_) => {
            return Err(Error::new(InvalidInput, "invalid input"));
        }
    }
}

pub fn tcp_read(stream: &mut TcpStream, buffer: &mut Vec<u8>) -> Result<usize, Error> {
    let mut size = [0u8; 4];
    stream.read_exact(&mut size)?;

    let len = u32::from_be_bytes(size) as usize;
    buffer.resize(len, 0);

    stream.read_exact(buffer)?;

    Ok(len)
}

pub fn tcp_write<T: Serialize>(stream: &mut TcpStream, data: T) -> Result<usize, Error> {
    let encoded = bincode::serialize(&data).unwrap();
    let len = encoded.len();
    let size = (len as u32).to_be_bytes(); // force 4 byte data length

    stream.write_all(&size)?;
    stream.write_all(&encoded)?;

    Ok(len)
}

#[macro_export]
macro_rules! config_dir {
    ($subpath: expr) => {{
        use directories::ProjectDirs;

        ProjectDirs::from("", "luftaquila", "transistor")
            .unwrap()
            .data_local_dir()
            .to_path_buf()
            .join($subpath)
    }};
}

