use std::net::UdpSocket;
use display_info::DisplayInfo;
use rdev::*;

const server_ip: &str = "127.0.0.1";
const port: u16 = 15007;

fn main() -> std::io::Result<()> {
    let display = DisplayInfo::all().unwrap();

    let socket = UdpSocket::bind("0.0.0.0:0")?;

    let hi = "hello";
    let result = socket.send_to(hi.as_bytes(), (server_ip, port));

    println!("res: {:?}", result);

    loop {
        // socket.recv_from

    }

    Ok(())

    // loop {
    //     if let Err(error) = listen(callback) {
    //         println!("Error: {:?}", error)
    //     }
    // }
}

fn config() {

}


// fn callback(event: Event) {
//     match event.event_type {
//         EventType::KeyPress(key) => {
//             println!("kepress: {:?}", event);
//         },

//         EventType::KeyRelease(key) => {
//             println!("keyrelease: {:?}", event);
//         },

//         EventType::ButtonPress(button) => {
//             println!("buttonpress: {:?}", event);
//         },

//         EventType::ButtonRelease(button) => {
//             println!("buttonrelease: {:?}", event);
//         },

//         EventType::MouseMove { x, y } => {
//             println!("mousemove: {:?}", event);
//         },

//         EventType::Wheel { delta_x, delta_y } => {
//             println!("wheel: {:?}", event);
//         }
//     }

//     // println!("My callback {:?}", event);
//     // match event.name {
//     //     Some(string) => println!("User wrote {:?}", string),
//     //     None => (),
//     // }
// }
