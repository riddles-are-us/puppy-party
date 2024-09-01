use std::collections::LinkedList;
use zkwasm_rest_abi::MERKLE_MAP;
use crate::config::{get_progress_increments};
use sha2::{Sha256, Digest};
use crate::player::{PLAYERLIST, Owner, PuppyPlayer};

#[derive(Clone)]
pub struct Event {
    pub owner: [u64; 4],
    pub current_action: u64
}

impl Event {
    fn compact(&self, buf: &mut Vec<u64>) {
        buf.push(self.owner[0]);
        buf.push(self.owner[1]);
        buf.push(self.owner[2]);
        buf.push(self.owner[3]);
        buf.push(self.current_action);
        zkwasm_rust_sdk::dbg!("compact {:?}", buf);
    }

    fn fetch(buf: &mut Vec<u64>) -> Event {
        zkwasm_rust_sdk::dbg!("fetch{:?}", buf);
        let current_action = buf.pop().unwrap();
        let mut owner = [
            buf.pop().unwrap(),
            buf.pop().unwrap(),
            buf.pop().unwrap(),
            buf.pop().unwrap(),
        ];
        owner.reverse();
        Event {
            owner,
            current_action
        }
    }
}

const SWAY: u64 = 0;
const LOTTERY: u64 = 6;

pub struct EventQueue {
    pub counter: u64,
    pub progress: u64,
    pub list: std::collections::LinkedList<Event>
}

impl EventQueue {
    // Helper function to process player ticks
    fn process_player_ticks(&self, player: &mut PuppyPlayer) {
        let pkey = PuppyPlayer::to_key(&player.player_id);

        // Decrease remaining_ticks by 1 and check for timeout
        if player.data.remaining_ticks > 0 {
            player.data.remaining_ticks -= 1;
            player.store();
        }
        if player.data.remaining_ticks == 0 {
            PLAYERLIST::new().delete_player(&pkey);
        }

        // Handle lottery timeout and reset
        if player.data.is_selected == 0 && player.data.lottery_ticks > 0 {
            player.data.lottery_ticks -= 1;
            player.store();
        }
        if player.data.is_selected == 0 && player.data.lottery_ticks == 0 {
            self.reset_lottery_state(player, &pkey);
        }
    }

    // Helper function to process player actions
    fn process_action(&self, owner_id: &[u64; 4], current_action: u64) {
        let mut player = PuppyPlayer::get(owner_id).unwrap();
        player.set_remaining_ticks(100);

        if current_action == LOTTERY {
            player.set_lottery_ticks(10);
        }

        player.store();
    }

    // Helper function to reset lottery state for a player
    fn reset_lottery_state(&self, player: &mut PuppyPlayer, pkey: &[u64;4]) {
        player.set_action(SWAY);
        player.set_is_selected(1);
        player.set_lottery_ticks(10);
        player.store();
        PLAYERLIST::new().store(pkey);
        let player_id = player.player_id;
        zkwasm_rust_sdk::dbg!("Player {:?} lottery canceled due to timeout.\n", player_id);
    }

    // Helper function to select a random player for the lottery
    fn select_random_player_for_lottery(&self, player_list: &mut PLAYERLIST) {
        let mut hasher = Sha256::new();
        hasher.update(&self.counter.to_le_bytes());
        let result: [u8; 32] = hasher.finalize().into();

        let random_index = (result[0] as usize) % player_list.0.len();
        let selected_player = player_list.0.get(random_index).unwrap();

        let pkey = PuppyPlayer::to_key(&selected_player.player_id);
        let mut player = PuppyPlayer::get(&pkey).unwrap();
        player.set_is_selected(0);
        player.store();
        PLAYERLIST::new().store(&pkey);
    }

    pub fn new() -> Self {
        EventQueue {
            counter: 0,
            progress: 0,
            list: LinkedList::new(),
        }
    }

    pub fn store(&self) {
        let n = self.list.len();
        let mut v = Vec::with_capacity(n * 5 + 1);
        for e in self.list.iter() {
            e.compact(&mut v);
        }
        v.push(self.counter);
        v.push(self.progress);
        let kvpair = unsafe { &mut MERKLE_MAP };
        kvpair.set(&[0, 0, 0, 0], v.as_slice());
        let root = kvpair.merkle.root.clone();
        zkwasm_rust_sdk::dbg!("root after store: {:?}\n", root);
    }

    pub fn fetch(&mut self) {
        let kvpair = unsafe { &mut MERKLE_MAP };
        let mut data = kvpair.get(&[0, 0, 0, 0]);
        if !data.is_empty() {
            let counter = data.pop().unwrap();
            let progress = data.pop().unwrap();
            let mut list = LinkedList::new();
            while !data.is_empty() {
                list.push_back(Event::fetch(&mut data))
            }
            self.counter = counter;
            self.progress = progress;
            self.list = list;
        }
    }

    pub fn dump(&self) {
        zkwasm_rust_sdk::dbg!("=-=-= dump queue =-=-=\n");
        for m in self.list.iter() {
            let owner = m.owner;
            zkwasm_rust_sdk::dbg!("{:?}\n", owner);
        }
        zkwasm_rust_sdk::dbg!("=-=-= end =-=-=\n");
    }

    pub fn tick(&mut self) {
        self.dump();
        let progress_increments = get_progress_increments();

        // Retrieve the player list
        let mut player_list = PLAYERLIST::get().unwrap();

        // Update progress and handle player ticks
        if self.list.is_empty() {
            self.progress += progress_increments.standard_increment;
            for player in player_list.0.iter_mut() {
                self.process_player_ticks(player);
            }
        } else {
            while let Some(head) = self.list.front_mut() {
                self.progress += progress_increments.action_reward;
                let owner_id = head.owner;
                let current_action = head.current_action;
                self.process_action(&owner_id, current_action);
                self.list.pop_front();
            }
        }

        self.counter += 1;

        // Select a random player to open a blind box if progress is sufficient
        if self.progress >= 100 {
            self.select_random_player_for_lottery(&mut player_list);
        }
    }

    pub fn insert(
      &mut self,
      owner: &[u64; 4],
      current_action: u64
    ) {
        let mut list = LinkedList::new();
        let node = Event {
            owner: owner.clone(),
            current_action
        };
        list.push_back(node);
        list.append(&mut self.list);
        self.list = list;
    }
}