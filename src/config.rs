use serde::Serialize;

const ACTIONS_SIZE: usize = 5;
const NAME_SIZE: usize = 5;

lazy_static::lazy_static! {
    pub static ref ADMIN_PUBKEY: [u64; 4] = {
        let bytes = include_bytes!("./admin.pubkey");
        // Interpret the bytes as an array of u64
        let u64s = unsafe { std::slice::from_raw_parts(bytes.as_ptr() as *const u64, 4) };
        u64s.try_into().unwrap()
    };
}

#[derive(Serialize, Clone)]
pub struct Config {
    actions: [&'static str; ACTIONS_SIZE],
    name: [&'static str; NAME_SIZE],
    action_reward: u32,
    action_duration: u64,
}

lazy_static::lazy_static! {
    pub static ref CONFIG: Config = Config {
        actions: ["shakeFeet", "shakeHead", "jump", "postComments", "lottery"],
        name: ["Bob", "Frank", "Cindy", "Alice", "John"],
        action_reward: 50,
        action_duration: 2
    };
}

impl Config {
    pub fn to_json_string() -> String {
        let meme_list: Vec<u64> = vec![];
        serde_json::to_string(&meme_list).unwrap()
    }

    // enable timer tick
    pub fn autotick() -> bool {
        true
    }
}

pub fn get_action_duration() -> u64 {
    CONFIG.action_duration
}

pub fn get_action_reward() -> u32 {
    CONFIG.action_reward
}
