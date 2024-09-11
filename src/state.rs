use crate::settlement::SettleMentInfo;
use std::cell::{RefCell, RefMut};
use crate::player::{PuppyPlayer, Owner};
use crate::Player;
use zkwasm_rest_abi::MERKLE_MAP;
use serde::{Serialize};
use crate::player::PlayerData;
use crate::config::{get_action_duration, get_action_reward};

#[derive(Serialize, Clone)]
pub struct QueryPlayerState {
    pid: String,
    data: PlayerData,
}

impl QueryPlayerState {
    fn from(p: &PuppyPlayer) -> Self {
        QueryPlayerState {
            pid: format!("{}-{}", p.player_id[0], p.player_id[1]),
            data: p.data.clone()
        }
    }
}

#[derive(Serialize)]
pub struct GlobalState {
    player_list: Vec<QueryPlayerState>,
    pub counter: u64
}

#[derive(Serialize)]
pub struct QueryState {
    player: PuppyPlayer,
    counter: u64,
    player_list: Vec<QueryPlayerState>,
}

impl GlobalState {
    pub fn new() -> Self {
        GlobalState {
            player_list: vec![],
            counter: 0,
        }
    }
    pub fn initialize() {
    }

    pub fn update_player_list(player: &mut PuppyPlayer, mut global_state: RefMut<'_, GlobalState>) {
        let player_data = QueryPlayerState::from(player);
        let mut exist = false;

        for p in global_state.player_list.iter_mut() {
            if p.pid == player_data.pid {
                *p = player_data.clone();
                exist = true;
            }
        }
        if exist == false {
            for p in global_state.player_list.iter_mut() {
              if p.data.progress < player_data.data.progress {
                  *p = player_data.clone();
              }
              break;
            }
        }
    }

    pub fn get_state(pid: Vec<u64>) -> String {
        let player = PuppyPlayer::get(&pid.try_into().unwrap()).unwrap();
        let player_list = GLOBAL_STATE.0.borrow().player_list.clone();
        let counter = GLOBAL_STATE.0.borrow().counter;
        serde_json::to_string({
            &QueryState {
                player,
                counter,
                player_list
            }
        }).unwrap()
    }

    pub fn preempt() -> bool {
        return false
    }

    pub fn flush_settlement() -> Vec<u8> {
        SettleMentInfo::flush_settlement()
    }

    pub fn rand_seed() -> u64 {
        0
    }

    pub fn store() {
    }
}

pub struct SafeState (RefCell<GlobalState>);
unsafe impl Sync for SafeState {}

lazy_static::lazy_static! {
    pub static ref GLOBAL_STATE: SafeState = SafeState(RefCell::new(GlobalState::new()));
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
const PLAYER_ACTION_NOT_FINISHED: u32 = 5;
const PLAYER_LOTTERY_EXPIRED: u32 = 6;
const PLAYER_LOTTERY_PROGRESS_NOT_FULL: u32 = 7;

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
           SELECTED_PLAYER_NOT_EXIST => "SelectedPlayerNotExist",
           PLAYER_ACTION_NOT_FINISHED => "PlayerActionNotFinished",
           PLAYER_LOTTERY_EXPIRED => "PlayerLotteryExpired",
           PLAYER_LOTTERY_PROGRESS_NOT_FULL => "PlayerLotteryProgressNotFull",
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
                0
            }
        }
    }

    pub fn action(&self, pkey: &[u64; 4], action: u64, rand: &[u64; 4]) -> u32 {
        let mut player = PuppyPlayer::get(pkey);
        let state = GLOBAL_STATE.0.borrow_mut();
        match player.as_mut() {
            None => ERROR_PLAYER_NOT_EXIST,
            Some(player) => {
                // Check for Lottery action
                if action == LOTTERY {
                    // This is the selected player; allow them to open the blind box
                    zkwasm_rust_sdk::dbg!("Player {:?} is opening the blind box", pkey);
                    if player.data.progress == 1000 {
                        if player.data.last_lottery_timestamp + 10 > state.counter {
                            // Update player's state to reflect that the lottery is complete
                            player.data.balance += 10; // change 10 to random reward
                            player.data.action = SWAY;
                            player.data.progress = 0;
                            player.data.last_lottery_timestamp = 0;
                            player.data.last_action_timestamp = 0;
                            player.check_and_inc_nonce(self.nonce);
                            player.store();
                            0
                        } else {
                            player.data.action = SWAY;
                            player.data.progress = 0;
                            player.data.last_lottery_timestamp = 0;
                            player.store();
                            PLAYER_LOTTERY_EXPIRED
                        }
                    } else {
                        PLAYER_LOTTERY_PROGRESS_NOT_FULL
                    }
                } else {
                    let action_duration = get_action_duration();
                    let action_reward = get_action_reward();

                    if player.data.last_action_timestamp != 0
                        && state.counter < player.data.last_action_timestamp + action_duration {
                        PLAYER_ACTION_NOT_FINISHED
                    } else {
                        player.data.action = action;
                        player.data.last_action_timestamp = state.counter;
                        player.data.progress += action_reward;
                        if player.data.progress == 1000 {
                            player.data.last_lottery_timestamp = state.counter;
                        } else if player.data.progress > 1000 {
                            player.data.progress = 1000;
                        }
                        GlobalState::update_player_list(player, state);
                        player.check_and_inc_nonce(self.nonce);
                        player.store();
                        0
                    }
                }
            }
        }
    }

    pub fn tick(&self) {
        GLOBAL_STATE.0.borrow_mut().counter += 1;
    }

    pub fn process(&self, pkey: &[u64; 4], rand: &[u64; 4]) -> u32 {
        let res = match self.command {
            CREATE_PLAYER => self.create_player(pkey),
            SHAKE_FEET => self.action(pkey, SHAKE_FEET, rand),
            JUMP => self.action(pkey, JUMP, rand),
            SHAKE_HEADS => self.action(pkey, SHAKE_HEADS, rand),
            POST_COMMENTS => self.action(pkey, POST_COMMENTS, rand),
            LOTTERY => self.action(pkey, LOTTERY, rand),
            _ => {
                self.tick();
                0
            }
        };
        let root = unsafe { &mut MERKLE_MAP.merkle.root };
        zkwasm_rust_sdk::dbg!("root after process {:?}\n", root);
        res
    }
}