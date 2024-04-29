use std::io::{Error, ErrorKind::*};

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
    pub name: String,
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

impl Display {
    pub fn from(item: DisplayInfo, cid: Cid) -> Self {
        Display {
            name: item.name,
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
            owner: cid,
        }
    }

    pub fn is_overlap(&self, target: Display) -> bool {
        let self_right = self.x + self.width;
        let self_bottom = self.y + self.height;
        let target_right = target.x + target.width;
        let target_bottom = target.y + target.height;

        self.x < target_right
            && self_right > target.x
            && self.y < target_bottom
            && self_bottom > target.y
    }

    pub fn is_touch(&self, target: Display) -> Option<(i32, i32, ZoneDirection)> {
        let self_right = self.x + self.width;
        let self_bottom = self.y + self.height;
        let target_right = target.x + target.width;
        let target_bottom = target.y + target.height;

        let horizontal_touch = (self_right == target.x || self.x == target_right)
            && (self.y < target_bottom && self_bottom > target.y);

        let vertical_touch = (self_bottom == target.y || self.y == target_bottom)
            && (self.x < target_right && self_right > target.x);

        if horizontal_touch {
            return Some((
                i32::max(self.y, target.y),
                i32::min(self_bottom, target_bottom),
                if self_right == target.x {
                    ZoneDirection::HorizontalRight
                } else {
                    ZoneDirection::HorizontalLeft
                },
            ));
        } else if vertical_touch {
            return Some((
                i32::max(self.x, target.x),
                i32::min(self_right, target_right),
                if self_bottom == target.y {
                    ZoneDirection::VerticalDown
                } else {
                    ZoneDirection::VerticalUp
                },
            ));
        } else {
            None
        }
    }
}

pub fn create_warpzones(a: &mut Vec<Display>, b: &mut Vec<Display>, eq: bool) -> Result<(), Error> {
    for (i, disp) in a.iter_mut().enumerate() {
        for (j, target) in b.iter_mut().enumerate() {
            if eq && i >= j {
                continue;
            }

            /* check overlapping */
            if disp.is_overlap(target.clone()) {
                return Err(Error::new(
                    InvalidInput,
                    "[ERR] two displays are overlapping",
                ));
            }

            /* add warpzone to each other if touching */
            if let Some((start, end, direction)) = disp.is_touch(target.clone()) {
                disp.warpzones.push(WarpZone {
                    start,
                    end,
                    direction,
                    to: target.owner,
                });

                target.warpzones.push(WarpZone {
                    start,
                    end,
                    direction: direction.reverse(),
                    to: disp.owner,
                });
            }
        }
    }

    Ok(())
}
