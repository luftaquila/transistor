use display_info::DisplayInfo;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SerializableDisplayInfo {
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
}

impl From<DisplayInfo> for SerializableDisplayInfo {
    fn from(item: DisplayInfo) -> Self {
        SerializableDisplayInfo {
            name: item.name,
            id: item.id,
            x: item.x,
            y: item.y,
            width: item.width,
            height: item.height,
            rotation: item.rotation,
            scale_factor: item.scale_factor,
            frequency: item.frequency,
            is_primary: item.is_primary,
        }
    }
}
