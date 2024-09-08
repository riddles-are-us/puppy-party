use std::collections::LinkedList;
use zkwasm_rest_abi::MERKLE_MAP;
use crate::config::get_progress_increments;
use crate::player::{Owner, PuppyPlayer};
use crate::reward::assign_reward_to_player;

const SWAY: u64 = 0;

#[derive(Clone)]
pub struct Event {
    pub owner: [u64; 4],
    pub delta: usize
}

impl Event {
    fn compact(&self, buf: &mut Vec<u64>) {
        buf.push(self.owner[0]);
        buf.push(self.owner[1]);
        buf.push(self.owner[2]);
        buf.push(self.owner[3]);
        buf.push(self.delta as u64);
        zkwasm_rust_sdk::dbg!("compact {:?}", buf);
    }

    fn fetch(buf: &mut Vec<u64>) -> Event {
        zkwasm_rust_sdk::dbg!("fetch{:?}", buf);
        let delta = buf.pop().unwrap();
        let mut owner = [
            buf.pop().unwrap(),
            buf.pop().unwrap(),
            buf.pop().unwrap(),
            buf.pop().unwrap(),
        ];
        owner.reverse();
        Event {
            owner,
            delta: delta as usize
        }
    }
}

pub struct EventQueue {
    pub counter: u64,
    pub list: std::collections::LinkedList<Event>
}

impl EventQueue {
    pub fn new() -> Self {
        EventQueue {
            counter: 0,
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
            let mut list = LinkedList::new();
            while !data.is_empty() {
                list.push_back(Event::fetch(&mut data))
            }
            self.counter = counter;
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

        while let Some(mut head) = self.list.pop_front() {
            if head.delta == 0 {
                self.list.pop_front();
            } else {
                let owner_id = head.owner;
                let mut player = PuppyPlayer::get(&owner_id).unwrap();
    
                // Decrease delta by 1 for every player
                head.delta -= 1;
                
                // Increase progress by standard_increment
                player.data.progress = player.data.progress + progress_increments.standard_increment;

                // Check lottery_ticks and update accordingly
                if player.data.reward != 0 {
                    if player.data.lottery_ticks == 0 {
                        player.data.reward = 0;
                        player.data.lottery_ticks = 10;
                        player.data.progress = 0;
                        player.data.action = SWAY;
                    }
                    player.data.lottery_ticks -= 1;
                }
    
                // Check if progress reached 1
                if player.data.progress >= 1 {
                    assign_reward_to_player(&mut player);
                }

                player.store();
            }  
        }
        self.counter += 1;
    }

    pub fn insert(
        &mut self,
        owner: &[u64; 4],
        delta: usize,
    ) {
        let mut list = LinkedList::new();
        let delta = delta;
        let mut found = false;

        // Search event with same owner
        while let Some(mut event) = self.list.pop_front() {
            // Reset delta to initial_delta(100)
            if event.owner == *owner {
                found = true;
                event.delta = 100;
            }

            list.push_back(event);
        }
    
        if !found {
            let node = Event {
                owner: owner.clone(),
                delta,
            };
            list.push_back(node);
        }
        self.list = list;
    }
}