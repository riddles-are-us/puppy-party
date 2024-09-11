use serde::Serialize;

const ACTIONS_SIZE: usize = 5;
const NAME_SIZE: usize = 5;

#[derive(Serialize, Clone)]
pub struct Config {
    actions: [&'static str; ACTIONS_SIZE],
    name: [&'static str; NAME_SIZE],
    action_reward: u64,
    action_duration: u64
}

lazy_static::lazy_static! {
    pub static ref CONFIG: Config = Config {
        actions: ["shakeFeet", "shakeHead", "jump", "postComments", "lottery"],
        name: ["Bob", "Frank", "Cindy", "Alice", "John"],
        action_reward: 50,
        action_duration: 3
    };


}

impl Config {
    pub fn to_json_string() -> String {
        serde_json::to_string(&CONFIG.clone()).unwrap()
    }

    // enable timer tick
    pub fn autotick() -> bool {
        true
    }
}

pub fn get_action_duration() -> u64 {
    CONFIG.action_duration
}

pub fn get_action_reward() -> u64 {
    CONFIG.action_reward
}