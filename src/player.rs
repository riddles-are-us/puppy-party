use crate::StorageData;
use serde::{Serialize};
use crate::Player;
use core::slice::IterMut;
use zkwasm_rest_abi::MERKLE_MAP;
use crate::config::player_list_key;

#[derive(Serialize)]
pub struct PLAYERLIST(Vec<PuppyPlayer>);

impl PLAYERLIST {
    pub fn new() -> Self {
        PLAYERLIST(Vec::new())
    }

    pub fn store(&self, player_id: &[u64; 4]) {
        let player = PuppyPlayer::get(player_id).unwrap();
        let kvpair = unsafe { &mut MERKLE_MAP };
        let plist_key = player_list_key();

        // Retrieve existing data from kvpair
        let data = kvpair.get(&plist_key);
        let data_len = data.len() as usize;

        // Prepare new data vector to hold updated player list
        let mut new_data = Vec::with_capacity(data.len() + player.player_id.len() + 4);
        let mut player_exists = false;
        //new_data.append(&mut data);

        // Iterate through existing players in the data
        let mut i = 0usize;

        while i < data_len {
            // Read player ID length
            let player_id_len = data[i] as usize;
            i += 1;

            // Extract player ID
            let current_player_id = &data[i..i + player_id_len];
            i += player_id_len;

            // Extract player nonce, name, and current_action
            let nonce = data[i];
            let name = data[i + 1];
            let current_action = data[i + 2];
            i += 3;

            // Check if this is the player we want to update
            if current_player_id == player.player_id {
                // Player exists; update their information
                player_exists = true;
                new_data.push(player_id_len as u64);
                new_data.extend_from_slice(current_player_id);
                new_data.push(player.nonce as u64);
                new_data.push(player.data.name as u64);
                new_data.push(player.data.current_action as u64);
            } else {
                // Copy existing player's data
                new_data.push(player_id_len as u64);
                new_data.extend_from_slice(current_player_id);
                new_data.push(nonce);
                new_data.push(name);
                new_data.push(current_action);
            }
        }

        // If the player did not exist, add them to the new data
        if !player_exists {
            new_data.push(player.player_id.len() as u64);
            for c in player.player_id.iter() {
                new_data.push(*c as u64);
            }
            new_data.push(player.nonce as u64);
            new_data.push(player.data.name as u64);
            new_data.push(player.data.current_action as u64);
        }

        // Store the updated player list back in kvpair
        kvpair.set(&plist_key, new_data.as_slice());
        zkwasm_rust_sdk::dbg!("end store player list\n");
    }

    pub fn get() -> Option<Self> {
        let kvpair = unsafe { &mut MERKLE_MAP };
        let plist_key = player_list_key();
        let data = kvpair.get(&plist_key);

        if data.is_empty() {
            None
        } else {
            let mut result = Vec::new();
            for chunk in data.chunks(6) {
                let player_id_size = chunk[0].clone();
                let mut player_id = Vec::new();
                for i in 0..player_id_size {
                    let index = (i + 1) as usize;
                    player_id.push(chunk[index] as u64);
                }
                let (_, rest) = chunk.split_at(player_id_size as usize + 1);
                let nonce = rest[0];
                let name = rest[1];
                let current_action = rest[2];
                let mut player = PuppyPlayer::get_from_pid(player_id.as_slice().try_into().unwrap()).unwrap();
                player.nonce = nonce;
                player.data.name = name;
                player.data.current_action = current_action;
                result.push(player);
            };
            Some(PLAYERLIST(result))
        }
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