use std::cell::RefCell;
use std::rc::Weak;

use display_info::DisplayInfo;
use serde::{Deserialize, Serialize};

use crate::client::*;

#[derive(Debug, Clone)]
enum ZoneDirection {
    HorizontalLeft,
    HorizontalRight,
    VerticalUp,
    VerticalDown,
}

#[derive(Debug, Clone)]
pub struct WarpZone {
    start: i32,
    end: i32,
    direction: ZoneDirection,
    to: Weak<RefCell<Display>>
}

#[derive(Debug, Clone, Default)]
enum DisplayOwnerType {
    SERVER,
    #[default]
    CLIENT,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Display {
    pub name: String,
    pub id: u32,
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
    // pub rotation: f32,
    // pub scale_factor: f32,
    // pub frequency: f32,
    // pub is_primary: bool,
    #[serde(skip)]
    warpzones: Vec<WarpZone>,
    #[serde(skip)]
    owner_type: DisplayOwnerType,
    #[serde(skip)]
    pub owner: Option<Weak<RefCell<Client>>>,
}

impl From<DisplayInfo> for Display {
    fn from(item: DisplayInfo) -> Self {
        Display {
            name: item.name,
            id: item.id,
            // without raw_handle
            x: item.x,
            y: item.y,
            width: item.width,
            height: item.height,
            // rotation: item.rotation,
            // scale_factor: item.scale_factor,
            // frequency: item.frequency,
            // is_primary: item.is_primary,
            warpzones: Vec::new(),
            owner_type: DisplayOwnerType::CLIENT,
            owner: None,
        }
    }
}
