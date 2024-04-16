use std::cell::RefCell;
use std::rc::{Rc, Weak};

use display_info::DisplayInfo;
use serde::{Deserialize, Serialize};

use crate::client::*;

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

#[derive(Debug, Clone)]
pub struct WarpZone {
    pub start: i32,
    pub end: i32,
    pub direction: ZoneDirection,
    pub to: Weak<RefCell<Display>>,
}

#[derive(Debug, Clone, Default)]
pub enum DisplayOwnerType {
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
    pub warpzones: Vec<WarpZone>,
    #[serde(skip)]
    pub owner_type: DisplayOwnerType,
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

impl Display {
    pub fn is_overlap(&self, target: &Rc<RefCell<Display>>) -> bool {
        let other = target.borrow();

        let self_right = self.x + self.width as i32;
        let self_bottom = self.y + self.height as i32;
        let other_right = other.x + other.width as i32;
        let other_bottom = other.y + other.height as i32;

        self.x < other_right
            && self_right > other.x
            && self.y < other_bottom
            && self_bottom > other.y
    }

    pub fn is_touch(&self, target: &Rc<RefCell<Display>>) -> Option<(i32, i32, ZoneDirection)> {
        let other = target.borrow();

        let self_right = self.x + self.width as i32;
        let self_bottom = self.y + self.height as i32;
        let other_right = other.x + other.width as i32;
        let other_bottom = other.y + other.height as i32;

        let horizontal_touch = (self_right == other.x || self.x == other_right)
            && (self.y < other_bottom && self_bottom > other.y);

        let vertical_touch = (self_bottom == other.y || self.y == other_bottom)
            && (self.x < other_right && self_right > other.x);

        if horizontal_touch {
            return Some((
                i32::max(self.y, other.y),
                i32::min(self_bottom, other_bottom),
                if self_right == other.x {
                    ZoneDirection::HorizontalRight
                } else {
                    ZoneDirection::HorizontalLeft
                },
            ));
        } else if vertical_touch {
            return Some((
                i32::max(self.x, other.x),
                i32::min(self_right, other_right),
                if self_bottom == other.y {
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
