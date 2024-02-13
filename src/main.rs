use display_info::DisplayInfo;
use rdev::*;

fn main() {
    let display_infos = DisplayInfo::all().unwrap();
    for display_info in display_infos {
        println!("display_info {display_info:?}");
    }

    // let socket = UdpSocket::bind("127.0.0.1:15007");

    loop {
        if let Err(error) = listen(callback) {
            println!("Error: {:?}", error)
        }
    }

}


fn callback(event: Event) {
    match event.event_type {
        EventType::KeyPress(key) => {
            println!("kepress: {:?}", event);
        },

        EventType::KeyRelease(key) => {
            println!("keyrelease: {:?}", event);
        },

        EventType::ButtonPress(button) => {
            println!("buttonpress: {:?}", event);
        },

        EventType::ButtonRelease(button) => {
            println!("buttonrelease: {:?}", event);
        },

        EventType::MouseMove { x, y } => {
            println!("mousemove: {:?}", event);
        },

        EventType::Wheel { delta_x, delta_y } => {
            println!("wheel: {:?}", event);
        }
    }

    // println!("My callback {:?}", event);
    // match event.name {
    //     Some(string) => println!("User wrote {:?}", string),
    //     None => (),
    // }
}
