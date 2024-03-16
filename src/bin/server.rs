use display_info::DisplayInfo;
use rdev::*;
use std::mem;
use std::net::{SocketAddr, UdpSocket};

const port: u16 = 15007;

fn main() {
    let display = DisplayInfo::all().unwrap();

    let socket = UdpSocket::bind(("127.0.0.1", port)).expect("bind failed!");

    // println!("waiting for client...");

    // let mut buf: [u8; 5] = [0; 5];
    // let (nb, src) = socket.recv_from(&mut buf).expect("handshake failed!");

    println!("start monitoring..");

    // monitor(socket, src);
    monitor(socket);
}

fn monitor(socket: UdpSocket /*, src: SocketAddr*/) {
    if let Err(error) = listen(move |event| {
        println!("event: {:?}, socket: {:?}", event, socket);
    }) {
        println!("input cature error: {:?}", error);
    }
}

fn callback(event: Event) {
    println!("event: {:?}", event);
    // let bytes: &[u8; mem::size_of::<Event>()];
    // let (nb, src) = socket.send_to(&mut buf).expect("config failed");
}

// fn config(socket: &UdpSocket, display: Vec<DisplayInfo>) -> Conf {
//     let mut buf = Conf { host_origin: Point { x: 10, y: 20 }, client_origin: Point { x: 30, y: 40 } };
//     let bytes: &[u8; mem::size_of::<Conf>()];
//     let (nb, src) = socket.recv_from(&mut buf).expect("config failed");

//     buf
// }
