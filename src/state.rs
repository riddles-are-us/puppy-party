use crate::events::EventQueue;
use std::cell::RefCell;
use crate::player::{PuppyPlayer, Owner};
use crate::Player;
use zkwasm_rest_abi::MERKLE_MAP;
use serde::{Serialize};
use crate::config::{get_initial_delta, get_progress_increments};

#[derive(Serialize)]
pub struct GlobalState {
    player: PuppyPlayer,
    counter: u64
}

impl GlobalState {
    pub fn initialize() {
    }

    pub fn get_state(pid: Vec<u64>) -> String {
        let player = PuppyPlayer::get(&pid.try_into().unwrap()).unwrap();
        let counter = QUEUE.0.borrow().counter;
        serde_json::to_string(&(player, counter)).unwrap()
    }

    pub fn store() {
    }
}

pub struct SafeEventQueue(RefCell<EventQueue>);
unsafe impl Sync for SafeEventQueue {}
impl SafeEventQueue {
    pub fn get_counter(&self) -> u64 {
        self.0.borrow().counter
    }
}

lazy_static::lazy_static! {
    pub static ref QUEUE: SafeEventQueue = SafeEventQueue (RefCell::new(EventQueue::new()));
}

const SWAY: u64 = 0;
const CREATE_PLAYER: u64 = 1;
const SHAKE_FEET: u64 = 2;
const JUMP: u64 = 3;
const SHAKE_HEADS: u64 = 4;
const POST_COMMENTS: u64 = 5;
const LOTTERY: u64 = 6;

const ERROR_PLAYER_ALREADY_EXIST:u32 = 1;
const ERROR_PLAYER_NOT_EXIST:u32 = 2;
const ERROR_NOT_SELECTED_PLAYER:u32 = 3;
const SELECTED_PLAYER_NOT_EXIST: u32 = 4;

pub struct Transaction {
    pub command: u64,
    pub nonce: u64
}

impl Transaction {
    pub fn decode_error(e: u32) -> &'static str {
        match e {
           ERROR_PLAYER_NOT_EXIST => "PlayerNotExist",
           ERROR_PLAYER_ALREADY_EXIST => "PlayerAlreadyExist",
           ERROR_NOT_SELECTED_PLAYER => "PlayerNotSelected",
           SELECTED_PLAYER_NOT_EXIST => "selectedPlayerNotExist",
           _ => "Unknown"
        }
    }

    pub fn decode(params: [u64; 4]) -> Self {
        let command = params[0] & 0xff;
        let nonce = params[0] >> 16;
        Transaction {
            command,
            nonce
        }
    }

    pub fn create_player(&self, pkey: &[u64; 4]) -> u32 {
        let player = PuppyPlayer::get(pkey);
        match player {
            Some(_) => ERROR_PLAYER_ALREADY_EXIST,
            None => {
                let player = Player::new(pkey);
                player.store();
                let initial_delta = get_initial_delta();
                QUEUE.0.borrow_mut().insert(pkey, initial_delta as usize);
                0
            }
        }
    }

    pub fn action(&self, pkey: &[u64; 4], action: u64) -> u32 {
        let mut player = PuppyPlayer::get(pkey);
        match player.as_mut() {
            None => ERROR_PLAYER_NOT_EXIST,
            Some(player) => {
                // Increase progress by action_reward when player run command
                let progress_increments = get_progress_increments();
();
                player.data.progress += progress_increments.action_reward;

                player.check_and_inc_nonce(self.nonce);

                // Reset delta as long as player run command
                let initial_delta = get_initial_delta();
                QUEUE.0.borrow_mut().insert(pkey, initial_delta as usize);

                // Check for Lottery action
                if action == LOTTERY {
                    // This is the selected player; allow them to open the blind box
                    zkwasm_rust_sdk::dbg!("Player {:?} is opening the blind box", pkey);

                    // Update player's state to reflect that the lottery is complete
                    player.data.reward = 0;
                    player.data.action = SWAY;
                    player.data.lottery_ticks = 10;
                    player.data.progress = 0;
                    player.store();

                    0
                } else {
                    player.data.action = action;
                    player.store();
                    0
                }
            }
        }
    }

    pub fn process(&self, pkey: &[u64; 4]) -> u32 {
        let res = match self.command {
            CREATE_PLAYER => self.create_player(pkey),
            SHAKE_FEET => self.action(pkey, SHAKE_FEET),
            JUMP => self.action(pkey, JUMP),
            SHAKE_HEADS => self.action(pkey, SHAKE_HEADS),
            POST_COMMENTS => self.action(pkey, POST_COMMENTS),
            LOTTERY => self.action(pkey, LOTTERY),
            _ => {
                QUEUE.0.borrow_mut().tick();
                0
            }
        };
        let root = unsafe { &mut MERKLE_MAP.merkle.root };
        zkwasm_rust_sdk::dbg!("root after process {:?}\n", root);
        res
    }
}