use display_info::DisplayInfo;
use enigo::*;

fn main() {
  let display_infos = DisplayInfo::all().unwrap();
  for display_info in display_infos {
    println!("display_info {display_info:?}");
  }

  while true {
      let mut enigo = Enigo::new();
      let cursor_location: (i32, i32) = enigo.mouse_location();
      println!("cursor_location {cursor_location:?}");
  }
}
