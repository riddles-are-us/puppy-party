use std::slice::IterMut;
use serde::Serialize;
use zkwasm_rest_abi::StorageData;
use zkwasm_rest_convention::{IndexedObject, Position};

#[derive(Clone, Serialize, Default, Copy)]
pub struct MemeInfo {
    pub id: u64,
    pub rank: u64,
    pub stake: u64,
    pub owner: [u64; 2],
}

impl StorageData for MemeInfo {
    fn from_data(u64data: &mut IterMut<u64>) -> Self {
        MemeInfo {
            id: *u64data.next().unwrap(),
            rank: *u64data.next().unwrap(),
            stake: *u64data.next().unwrap(),
            owner: [*u64data.next().unwrap(),*u64data.next().unwrap()],
        }
    }
    fn to_data(&self, data: &mut Vec<u64>) {
        data.push(self.id);
        data.push(self.rank);
        data.push(self.stake);
        data.push(self.owner[0]);
        data.push(self.owner[1]);
    }
}

#[derive(Clone, Serialize, Default, Copy, Debug)]
pub struct StakeInfo {
    pub stake: u64,
    pub timestamp: u64 // last time the user collects their rewards
}

impl StorageData for StakeInfo {
    fn from_data(u64data: &mut IterMut<u64>) -> Self {
        StakeInfo {
            stake: *u64data.next().unwrap(),
            timestamp: *u64data.next().unwrap(),
        }
    }
    fn to_data(&self, data: &mut Vec<u64>) {
        data.push(self.stake);
        data.push(self.timestamp);
    }
}


impl IndexedObject<MemeInfo> for MemeInfo {
    const PREFIX: u64 = 0x1ee1;
    const POSTFIX: u64 = 0xfee1;
    const EVENT_NAME: u64 = 0x02;
}

impl Position<StakeInfo> for StakeInfo {
    const PREFIX: u64 = 0x1ff1;
    const POSTFIX: u64 = 0xf1f1;
    const EVENT_NAME: u64 = 0x01;
}
