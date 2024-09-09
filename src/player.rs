use crate::StorageData;
use serde::{Serialize};
use core::slice::IterMut;
use crate::Player;

#[derive(Clone, Serialize, Debug)]
pub struct PlayerData {
    pub name: u64, // index of name in config
    pub action: u64,
    pub lottery_ticks: u64, // Starts at 10
    pub reward: u64,
    pub progress: u64
}

impl Default for PlayerData {
    fn default() -> Self {
        Self {
            name: 0,
            action: 0,
            lottery_ticks: 10,
            reward: 0,
            progress: 0
        }
    }
}

impl StorageData for PlayerData {
    fn from_data(u64data: &mut IterMut<u64>) -> Self {
        PlayerData {
            name: *u64data.next().unwrap(),
            action: *u64data.next().unwrap(),
            lottery_ticks: *u64data.next().unwrap(),
            reward: *u64data.next().unwrap(),
            progress: *u64data.next().unwrap()
        }
    }
    fn to_data(&self, data: &mut Vec<u64>) {
        data.push(self.name);
        data.push(self.action);
        data.push(self.lottery_ticks);
        data.push(self.reward);
        data.push(self.progress);
    }
}

pub type PuppyPlayer = Player<PlayerData>;

pub trait Owner: Sized {
    fn new(pkey: &[u64; 4]) -> Self;
    fn get(pkey: &[u64; 4]) -> Option<Self>;
}

impl Owner for PuppyPlayer {
    fn new(pkey: &[u64; 4]) -> Self {
        Self::new_from_pid(Self::pkey_to_pid(pkey))
    }

    fn get(pkey: &[u64; 4]) -> Option<Self> {
        Self::get_from_pid(&Self::pkey_to_pid(pkey))
    }
}