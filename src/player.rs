use crate::StorageData;
use serde::{Serialize};
use crate::Player;
use core::slice::IterMut;
use zkwasm_rest_abi::MERKLE_MAP;
use crate::config::player_list_key;

#[derive(Serialize)]
pub struct PLAYERLIST(pub Vec<PuppyPlayer>);

impl PLAYERLIST {
    pub fn new() -> Self {
        PLAYERLIST(Vec::new())
    }

    // A utility function to modify players in the list
    fn modify_players<F>(&self, mut modify_fn: F) -> Vec<u64>
    where
        F: FnMut(&[u64], u64, u64, u64, u64, u64) -> Option<Vec<u64>>,
    {
        let kvpair = unsafe { &mut MERKLE_MAP };
        let plist_key = player_list_key();
        let data = kvpair.get(&plist_key);
        let data_len = data.len() as usize;

        // Prepare new data vector to hold updated player list
        let mut new_data = Vec::with_capacity(data.len());

        // Iterate through existing players in the data
        let mut i = 0usize;
        while i < data_len {
            let player_id_len = data[i] as usize;
            i += 1;

            let current_player_id = &data[i..i + player_id_len];
            i += player_id_len;

            let nonce = data[i];
            let name = data[i + 1];
            let current_action = data[i + 2];
            let is_selected = data[i + 3];
            let remaining_ticks = data[i + 4];
            i += 5;

            // Use closure to determine the operation on each player
            if let Some(updated_player_data) = modify_fn(
                current_player_id,
                nonce,
                name,
                current_action,
                is_selected,
                remaining_ticks
            ) {
                new_data.extend(updated_player_data);
            }
        }

        new_data
    }

    // Store player in the list
    pub fn store(&self, player_id: &[u64; 4]) {
        let kvpair = unsafe { &mut MERKLE_MAP };
        let player = PuppyPlayer::get(player_id).unwrap();
        let plist_key = player_list_key();
        let mut player_exists = false;

        let mut new_data = self.modify_players(|current_player_id, nonce, name, current_action, is_selected, remaining_ticks| {
            let mut result = Vec::new();

            if current_player_id == player_id {
                // Update the player's information
                player_exists = true;
                result.push(player_id.len() as u64);
                result.extend_from_slice(current_player_id);
                result.push(player.nonce as u64);
                result.push(player.data.name as u64);
                result.push(player.data.current_action as u64);
                result.push(player.data.is_selected as u64);
                result.push(player.data.remaining_ticks as u64);
            } else {
                // Keep the current player's data unchanged
                result.push(current_player_id.len() as u64);
                result.extend_from_slice(current_player_id);
                result.push(nonce);
                result.push(name);
                result.push(current_action);
                result.push(is_selected);
                result.push(remaining_ticks);
            }
            Some(result)
        });

         // If the player did not exist, add them to the new data
         if !player_exists {
            new_data.push(player.player_id.len() as u64);
            for c in player.player_id.iter() {
                new_data.push(*c as u64);
            }
            new_data.push(player.nonce as u64);
            new_data.push(player.data.name as u64);
            new_data.push(player.data.current_action as u64);
            new_data.push(player.data.is_selected as u64);
            new_data.push(player.data.remaining_ticks as u64);
        }

        // Store the updated player list back in kvpair
        kvpair.set(&plist_key, new_data.as_slice());

        zkwasm_rust_sdk::dbg!("end store player list\n");
    }

    // Delete a player from the list
    pub fn delete_player(&self, player_id: &[u64; 4]) {
        let kvpair = unsafe { &mut MERKLE_MAP };
        let plist_key = player_list_key();

        let new_data = self.modify_players(|current_player_id, nonce, name, current_action, is_selected, remaining_ticks| {
            if current_player_id == player_id {
                // Return None to indicate player deletion
                None
            } else {
                // Keep the current player's data unchanged
                let mut result = Vec::new();
                result.push(current_player_id.len() as u64);
                result.extend_from_slice(current_player_id);
                result.push(nonce);
                result.push(name);
                result.push(current_action);
                result.push(is_selected);
                result.push(remaining_ticks);
                Some(result)
            }
        });

        // Store the updated player list back in kvpair
        kvpair.set(&plist_key, new_data.as_slice());

        zkwasm_rust_sdk::dbg!("Player deleted, updated player list stored.\n");
    }

    pub fn get() -> Option<Self> {
        let kvpair = unsafe { &mut MERKLE_MAP };
        let plist_key = player_list_key();
        let data = kvpair.get(&plist_key);

        if data.is_empty() {
            Some(PLAYERLIST::new())
        } else {
            let mut result = Vec::new();
            for chunk in data.chunks(8) {
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
                let is_selected = rest[2];
                let remaining_ticks = rest[2];
                let mut player = PuppyPlayer::get_from_pid(player_id.as_slice().try_into().unwrap()).unwrap();
                player.nonce = nonce;
                player.data.name = name;
                player.data.current_action = current_action;
                player.data.is_selected = is_selected;
                player.data.remaining_ticks = remaining_ticks;
                result.push(player);
            };
            Some(PLAYERLIST(result))
        }
    }
}

#[derive(Clone, Serialize, Debug)]
pub struct PlayerData {
    pub name: u64, // index of name in config
    pub current_action: u64,
    pub is_selected: u64, // 0: true, 1: false
    pub remaining_ticks: u64, // Starts at 100
    pub lottery_ticks: u64, // Starts at 10
    pub reward: u64
}

impl Default for PlayerData {
    fn default() -> Self {
        Self {
            name: 0,
            current_action: 0,
            is_selected: 1,
            remaining_ticks: 100,
            lottery_ticks: 10,
            reward: 0
        }
    }
}

impl StorageData for PlayerData {
    fn from_data(u64data: &mut IterMut<u64>) -> Self {
        PlayerData {
            name: *u64data.next().unwrap(),
            current_action: *u64data.next().unwrap(),
            is_selected: *u64data.next().unwrap(),
            remaining_ticks: *u64data.next().unwrap(),
            lottery_ticks: *u64data.next().unwrap(),
            reward: *u64data.next().unwrap()
        }
    }
    fn to_data(&self, data: &mut Vec<u64>) {
        data.push(self.name);
        data.push(self.current_action);
        data.push(self.is_selected);
        data.push(self.remaining_ticks);
        data.push(self.lottery_ticks);
        data.push(self.reward);
    }
}

pub type PuppyPlayer = Player<PlayerData>;

pub trait Owner: Sized {
    fn new(pkey: &[u64; 4]) -> Self;
    fn get(pkey: &[u64; 4]) -> Option<Self>;
    fn set_name(&mut self, name: u64);
    fn set_action(&mut self, action_index: u64);
    fn set_is_selected(&mut self, is_selected: u64);
    fn set_remaining_ticks(&mut self, remaining_ticks: u64);
    fn set_lottery_ticks(&mut self, lottery_ticks: u64);
    fn set_reward(&mut self, reward: u64);
}

impl Owner for PuppyPlayer {
    fn new(pkey: &[u64; 4]) -> Self {
        Self::new_from_pid(Self::pkey_to_pid(pkey))
    }

    fn get(pkey: &[u64; 4]) -> Option<Self> {
        Self::get_from_pid(&Self::pkey_to_pid(pkey))
    }

    fn set_name(&mut self, name: u64) {
        self.data.name = name;
    }

    fn set_action(&mut self, action_index: u64) {
        self.data.current_action = action_index;
    }

    fn set_is_selected(&mut self, is_selected: u64) {
        self.data.is_selected = is_selected;
    }

    fn set_remaining_ticks(&mut self, remaining_ticks: u64) {
        self.data.remaining_ticks = remaining_ticks;
    }

    fn set_lottery_ticks(&mut self, lottery_ticks: u64) {
        self.data.lottery_ticks = lottery_ticks;
    }

    fn set_reward(&mut self, reward: u64) {
        self.data.reward = reward;
    }
}