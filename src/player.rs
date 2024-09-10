use crate::StorageData;
use serde::{Serialize};
use core::slice::IterMut;
use crate::Player;

#[derive(Clone, Serialize, Debug)]
pub struct PlayerData {
    pub balance: u64,
    pub action: u64,
    pub last_lottery_timestamp: u64, // last timestamp when this user allowed to pick a lottery
    pub last_action_timestamp: u64, // last timestamp when this user allowed to pick a lottery
    pub progress: u64
}

impl Default for PlayerData {
    fn default() -> Self {
        Self {
            action: 0,
            last_lottery_timestamp: 0,
            last_action_timestamp: 0, // last timestamp when this user allowed to pick a lottery
            balance: 0,
            progress: 0
        }
    }
}

impl StorageData for PlayerData {
    fn from_data(u64data: &mut IterMut<u64>) -> Self {
        PlayerData {
            action: *u64data.next().unwrap(),
            last_lottery_timestamp: *u64data.next().unwrap(),
            last_action_timestamp: *u64data.next().unwrap(),
            balance: *u64data.next().unwrap(),
            progress: *u64data.next().unwrap()
        }
    }
    fn to_data(&self, data: &mut Vec<u64>) {
        data.push(self.action);
        data.push(self.last_lotter_timestamp);
        data.push(self.last_action_timestamp);
        data.push(self.balance);
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
