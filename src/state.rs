use crate::{player::PLAYERLIST};
use crate::events::EventQueue;
use std::cell::RefCell;
use crate::player::{PuppyPlayer};
use crate::Player;
use zkwasm_rest_abi::MERKLE_MAP;
use serde::{Serialize};
use crate::player::Owner;

#[derive(Serialize)]
pub struct GlobalState {
    player_list: PLAYERLIST,
    progress: u64,
    counter: u64
}

impl GlobalState {
    pub fn initialize() {
    }

    pub fn get_state(_pid: Vec<u64>) -> String {
        let player_list = PLAYERLIST::get().unwrap();
        let progress = QUEUE.0.borrow().progress;
        let counter = QUEUE.0.borrow().counter;
        serde_json::to_string(&(player_list, progress, counter)).unwrap()
    }

    pub fn store() {
    }
}

pub struct SafeEventQueue(RefCell<EventQueue>);
unsafe impl Sync for SafeEventQueue {}

lazy_static::lazy_static! {
    pub static ref QUEUE: SafeEventQueue = SafeEventQueue (RefCell::new(EventQueue::new()));
}

const CREATE_PLAYER: u64 = 1;
const SHAKE_FEET: u64 = 2;
const JUMP: u64 = 3;
const SHAKE_HEADS: u64 = 4;
const POST_COMMENTS: u64 = 5;
const LOTTERY: u64 = 6;

const ERROR_PLAYER_ALREADY_EXIST:u32 = 1;
const ERROR_PLAYER_NOT_EXIST:u32 = 2;

pub struct Transaction {
    pub command: u64,
    pub nonce: u64,
    pub data: Vec<u64>
}

impl Transaction {
    pub fn decode_error(e: u32) -> &'static str {
        match e {
           ERROR_PLAYER_NOT_EXIST => "PlayerNotExist",
           ERROR_PLAYER_ALREADY_EXIST => "PlayerAlreadyExist",
           _ => "Unknown"
        }
    }

    pub fn decode(params: [u64; 4]) -> Self {
        let command = params[0] & 0xff;
        let nonce = params[0] >> 16;
        let data = vec![params[1], params[2], params[3]];
        Transaction {
            command,
            nonce,
            data
        }
    }

    pub fn create_player(&self, pkey: &[u64; 4]) -> u32 {
        let player = PuppyPlayer::get(pkey);
        match player {
            Some(_) => ERROR_PLAYER_ALREADY_EXIST,
            None => {
                let player = Player::new(&pkey);
                player.store();
                PLAYERLIST::new().store(pkey);
                0
            }
        }
    }

    pub fn action(&self, pkey: &[u64; 4], action: u64) -> u32 {
        let mut player = PuppyPlayer::get(pkey);
        match player.as_mut() {
            None => ERROR_PLAYER_NOT_EXIST,
            Some(player) => {
                player.check_and_inc_nonce(self.nonce);
                player.set_action(action);
                player.store();
                QUEUE.0.borrow_mut().insert(0);
                0
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