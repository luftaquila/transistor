use serde::{Deserialize, Serialize};

use crate::display::*;

#[derive(Serialize, Deserialize, Debug)]
pub enum Action {
    Warp,
    Move,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Message {
    pub disp: Did,
    pub action: Action,
    pub x: i32,
    pub y: i32,
}

