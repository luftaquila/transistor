use std::collections::HashMap;

use display_info::DisplayInfo;
use mouce::Mouse;

use crate::Client;
use crate::Display;

pub struct Server {
    displays: Vec<Display>,
    clients: HashMap<String, Client>
}
