use display_info::DisplayInfo;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub enum HandshakeStatus {
    HandshakeOk,
    HandshakeErr,
}

pub fn print_displays() {
    println!("[INF] detected system displays:");
    let displays = DisplayInfo::all().unwrap();

    for display in displays {
        println!("  {:?}", display);
    }
}

#[macro_export]
macro_rules! config_dir {
    () => {{
        use directories::ProjectDirs;

        ProjectDirs::from("", "luftaquila", "transistor")
            .unwrap()
            .data_local_dir()
            .to_path_buf()
    }};
}

#[macro_export]
macro_rules! tcp_stream_read {
    ($stream:expr, $buffer:expr) => {{
        let mut size = [0u8; 4];
        $stream.read_exact(&mut size)?;

        let len = u32::from_be_bytes(size) as usize;
        $stream.read_exact(&mut $buffer[..len])?;

        len
    }};
}

#[macro_export]
macro_rules! tcp_stream_read_resize {
    ($stream:expr, $buffer:expr) => {{
        let mut size = [0u8; 4];
        $stream.read_exact(&mut size)?;

        let len = u32::from_be_bytes(size) as usize;
        $buffer.resize(len, 0);
        $stream.read_exact(&mut $buffer)?;

        len
    }};
}

#[macro_export]
macro_rules! tcp_stream_write {
    ($stream:expr, $data:expr) => {
        let encoded = bincode::serialize(&$data).unwrap();

        /* force 4 byte data length */
        let len = encoded.len() as u32;
        let size = len.to_be_bytes();

        if let Err(e) = $stream.write_all(&size) {
            eprintln!("[ERR] TCP stream write failed: {}", e);
        }

        if let Err(e) = $stream.write_all(&encoded) {
            eprintln!("[ERR] TCP stream write failed: {}", e);
        }
    };
}
