mod client;
mod display;
mod server;
mod utils;

pub use client::*;
pub use display::*;
pub use server::*;
pub use utils::*;

pub const PORT: u16 = 2426;
pub const SERVER_CID: Cid = 0;
pub const MARGIN: i32 = 2;
