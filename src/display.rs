use display_info::DisplayInfo;
use rand;
use serde::{Deserialize, Serialize};

use crate::Cid;

pub type Did = u32;

#[derive(Debug, Clone, Copy)]
pub enum ZoneDirection {
    HorizontalLeft,
    HorizontalRight,
    VerticalUp,
    VerticalDown,
}

impl ZoneDirection {
    pub fn reverse(&self) -> Self {
        match self {
            Self::HorizontalLeft => Self::HorizontalRight,
            Self::HorizontalRight => Self::HorizontalLeft,
            Self::VerticalUp => Self::VerticalDown,
            Self::VerticalDown => Self::VerticalUp,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct WarpZone {
    pub start: i32,
    pub end: i32,
    pub direction: ZoneDirection,
    pub to: Cid,
}

#[derive(Debug)]
pub struct AssignedDisplays {
    pub system: Vec<Did>,
    pub client: Vec<Did>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Display {
    pub id: Did,
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
    pub rotation: f32,
    pub scale_factor: f32,
    pub frequency: f32,
    pub is_primary: bool,
    #[serde(skip)]
    pub warpzones: Vec<WarpZone>,
    pub owner: Cid,
}

impl From<DisplayInfo> for Display {
    fn from(item: DisplayInfo) -> Self {
        Display {
            // name - not meaningful on every platform
            id: rand::random(),
            // raw_handle - cannot serialize
            x: item.x,
            y: item.y,
            width: item.width as i32,
            height: item.height as i32,
            rotation: item.rotation,
            scale_factor: item.scale_factor,
            frequency: item.frequency,
            is_primary: item.is_primary,
            warpzones: Vec::new(),
            owner: 0,
        }
    }
}
