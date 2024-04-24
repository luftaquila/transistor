use display_info::DisplayInfo;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Display {
    pub name: String,
    pub id: u32,
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
    pub rotation: f32,
    pub scale_factor: f32,
    pub frequency: f32,
    pub is_primary: bool,
    // #[serde(skip)]
    // pub warpzones: Vec<WarpZone>,
    pub owner: String,
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
            rotation: item.rotation,
            scale_factor: item.scale_factor,
            frequency: item.frequency,
            is_primary: item.is_primary,
            // warpzones: Vec::new(),
            owner: String::new(),
        }
    }
}
