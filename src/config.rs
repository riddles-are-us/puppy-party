use serde::Serialize;

const ACTIONS_SIZE: usize = 5;
const NAME_SIZE: usize = 5;

#[derive(Serialize, Clone)]
pub struct ProgressIncrements {
    pub standard_increment: u64,
    pub action_reward: u64,
}

#[derive(Serialize, Clone)]
pub struct Config {
    actions: [&'static str; ACTIONS_SIZE],
    name: [&'static str; NAME_SIZE],
    pub progress_increments: ProgressIncrements,
    initial_delta: u64
}

lazy_static::lazy_static! {
    pub static ref CONFIG: Config = Config {
        actions: ["shakeFeet", "shakeHead", "jump", "postComments", "lottery"],
        name: ["Bob", "Frank", "Cindy", "Alice", "John"],
        progress_increments: ProgressIncrements {
            standard_increment: 1,
            action_reward: 50
        },
        initial_delta: 100
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

pub fn get_progress_increments() -> &'static ProgressIncrements {
    &CONFIG.progress_increments
}

pub fn get_initial_delta() -> u64 {
    CONFIG.initial_delta
}
