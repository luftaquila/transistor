mod utils;
mod display;
mod client;
mod server;

pub use utils::*;
pub use display::*;
pub use client::*;
pub use server::*;

pub const PORT: u16 = 2426;
pub const MAX_DISPLAYS: u32 = 128;
