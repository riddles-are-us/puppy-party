use crate::StorageData;
use serde::{Serialize};
use crate::Player;
use core::slice::IterMut;
use zkwasm_rest_abi::MERKLE_MAP;

#[derive(Serialize)]
pub struct PLAYERLIST(Vec<PuppyPlayer>);

impl PLAYERLIST {
    pub fn new() -> Self {
        PLAYERLIST(Vec::new())
    }

    pub fn store(&self, player_id: &[u64; 4]) {
        let player = PuppyPlayer::get(player_id).unwrap();
        let kvpair = unsafe { &mut MERKLE_MAP };
        let mut data = kvpair.get(&[1,1,1,1]);
        let mut new_data = Vec::with_capacity(data.len() + player.player_id.len() + 4);

        new_data.append(&mut data);

        // append player info
        new_data.push(player.player_id.len() as u64);
        for c in player.player_id.iter() {
            new_data.push(*c as u64);
        }
        new_data.push(player.nonce as u64);
        new_data.push(player.data.name as u64);
        new_data.push(player.data.current_action as u64);

        let kvpair = unsafe { &mut MERKLE_MAP };
        kvpair.set(&[1,1,1,1], new_data.as_slice());
        zkwasm_rust_sdk::dbg!("end store player list\n");
    }

    pub fn get() -> Option<Self> {
        let kvpair = unsafe { &mut MERKLE_MAP };
        let data = kvpair.get(&[1,1,1,1]);
        if data.is_empty() {
            None
        } else {
            let mut result = Vec::new();
            for chunk in data.chunks(6) {
                let player_id_size = chunk[0].clone();
                let mut player_id = Vec::new();
                for i in 0..player_id_size {
                    let index = (i + 1) as usize;
                    player_id.push(chunk[index]);
                }
                let (_, rest) = chunk.split_at(player_id_size as usize + 1);
                let nonce = rest[0];
                let name = rest[1];
                let current_action = rest[2];
                let mut player = PuppyPlayer::get(&player_id.try_into().unwrap()).unwrap();
                player.nonce = nonce;
                player.data.name = name;
                player.data.current_action = current_action;
                result.push(player);
            }
            Some(PLAYERLIST(result))        }
    }
}

#[derive(Clone, Serialize, Debug)]
pub struct PlayerData {
    pub name: u64,
    pub current_action: u64
}

impl Default for PlayerData {
    fn default() -> Self {
        Self {
            name: 0,
            current_action: 0
        }
    }
}

impl StorageData for PlayerData {
    fn from_data(u64data: &mut IterMut<u64>) -> Self {
        PlayerData {
            name: *u64data.next().unwrap(),
            current_action: *u64data.next().unwrap()
        }
    }
    fn to_data(&self, data: &mut Vec<u64>) {
        data.push(self.name);
        data.push(self.current_action);
    }
}

pub type PuppyPlayer = Player<PlayerData>;

pub trait Owner: Sized {
    fn new(pkey: &[u64; 4]) -> Self;
    fn get(pkey: &[u64; 4]) -> Option<Self>;
    fn set_action(&mut self, action_index: u64);
}

impl Owner for PuppyPlayer {
    fn new(pkey: &[u64; 4]) -> Self {
        Self::new_from_pid(Self::pkey_to_pid(pkey))
    }

    fn get(pkey: &[u64; 4]) -> Option<Self> {
        Self::get_from_pid(&Self::pkey_to_pid(pkey))
    }

    fn set_action(&mut self, action_index: u64) {
        self.data.current_action = action_index;
    }
}