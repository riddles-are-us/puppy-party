use serde::Serialize;
use crate::settlement::SettleMentInfo;

const ACTIONS_SIZE: usize = 5;
const NAME_SIZE: usize = 5;

#[derive(Serialize, Clone)]
pub struct Config {
    actions: [&'static str; ACTIONS_SIZE],
    name: [&'static str; NAME_SIZE]
}

lazy_static::lazy_static! {
    pub static ref CONFIG: Config = Config {
        actions: ["shakeFeet", "shakeHead", "jump", "postComments", "lottery"],
        name: ["Bob", "Frank", "Cindy", "Alice", "John"]
    };
}

impl Config {
    pub fn to_json_string() -> String {
        serde_json::to_string(&CONFIG.clone()).unwrap()
    }

    pub fn flush_settlement() -> Vec<u8> {
        SettleMentInfo::flush_settlement()
    }
}