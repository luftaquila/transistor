use std::path::PathBuf;

use directories::ProjectDirs;
use display_info::DisplayInfo;

pub fn config_dir() -> PathBuf {
    ProjectDirs::from("io", "luftaquila", "transistor")
        .unwrap()
        .data_local_dir()
        .to_path_buf()
}

pub fn print_displays() {
    println!("[INF] detected system displays:");
    let displays = DisplayInfo::all().unwrap();

    for display in displays {
        println!("  {:?}", display);
    }
}

