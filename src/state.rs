use crate::settlement::SettlementInfo;
use std::cell::{RefCell};
use crate::player::{PuppyPlayer, Owner};
use crate::Player;
use zkwasm_rest_abi::{ MERKLE_MAP, WithdrawInfo };
use serde::{Serialize, Serializer, ser::SerializeSeq};
use core::slice::IterMut;
use crate::player::PlayerData;
use crate::config::{get_action_duration, get_action_reward};
use zkwasm_rust_sdk::require;
use zkwasm_rest_abi::StorageData;

// Custom serializer for `[u64; 2]` as a [String; 2].
fn serialize_u64_array_as_string<S>(value: &[u64; 2], serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(value.len()))?;
        for e in value.iter() {
            seq.serialize_element(&e.to_string())?;
        }
        seq.end()
    }


#[derive(Serialize, Clone)]
pub struct QueryPlayerState {
    #[serde(serialize_with="serialize_u64_array_as_string")]
    pid: [u64;2],
    data: PlayerData,
}

impl QueryPlayerState {
    fn from(p: &PuppyPlayer) -> Self {
        QueryPlayerState {
            pid: [p.player_id[0], p.player_id[1]],
            data: p.data.clone()
        }
    }
}

impl StorageData for QueryPlayerState {

    fn to_data(&self, buf: &mut Vec<u64>) {
        buf.push(self.pid[0]);
        buf.push(self.pid[1]);
        buf.push(self.data.action);
        buf.push(self.data.last_lottery_timestamp);
        buf.push(self.data.last_action_timestamp);
        buf.push(self.data.balance);
        buf.push(self.data.progress);
    }

    fn from_data(u64data: &mut IterMut<u64>) -> QueryPlayerState {
        let pid = [
            *u64data.next().unwrap(),
            *u64data.next().unwrap()
        ];

        let action = *u64data.next().unwrap();
        let last_lottery_timestamp = *u64data.next().unwrap();
        let last_action_timestamp = *u64data.next().unwrap();
        let balance = *u64data.next().unwrap();
        let progress = *u64data.next().unwrap();
        QueryPlayerState {
            pid,
            data: PlayerData {
                action,
                last_lottery_timestamp,
                last_action_timestamp,
                balance,
                progress
            }
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

    pub fn update_player_list(player: &mut PuppyPlayer) {
        let player_data = QueryPlayerState::from(player);
        let mut global_state = GLOBAL_STATE.0.borrow_mut();
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

    pub fn snapshot() -> String {
        let player_list = GLOBAL_STATE.0.borrow().player_list.clone();
        let counter = GLOBAL_STATE.0.borrow().counter;
        serde_json::to_string({
            &QueryState {
                counter,
                player_list
            }
        }).unwrap()
    }

    pub fn get_state(pid: Vec<u64>) -> String {
        let player = PuppyPlayer::get(&pid.try_into().unwrap()).unwrap();
        serde_json::to_string(&player).unwrap()
    }

    pub fn preempt() -> bool {
        let counter = GLOBAL_STATE.0.borrow().counter;
        if counter % 30 == 0 {
            return true
        } else {
            return false
        }
    }

    pub fn flush_settlement() -> Vec<u8> {
        SettlementInfo::flush_settlement()
    }

    pub fn rand_seed() -> u64 {
        0
    }

    pub fn store_into_kvpair(&self) {
        let n = self.player_list.len();
        let mut v = Vec::with_capacity(n * 7 + 1);
        v.push(self.counter);
        for e in self.player_list.iter() {
            e.to_data(&mut v);
        }
        let kvpair = unsafe { &mut MERKLE_MAP };
        kvpair.set(&[0, 0, 0, 0], v.as_slice());
        let root = kvpair.merkle.root.clone();
        zkwasm_rust_sdk::dbg!("root after store: {:?}\n", root);
    }

    pub fn fetch(&mut self) {
        let kvpair = unsafe { &mut MERKLE_MAP };
        let mut data = kvpair.get(&[0, 0, 0, 0]);
        if !data.is_empty() {
            let mut u64data = data.iter_mut();
            let counter = *u64data.next().unwrap();
            let mut player_list = vec![];
            while u64data.len() != 0 {
                player_list.push(QueryPlayerState::from_data(&mut u64data))
            }
            self.counter = counter;
            self.player_list = player_list;
        }
    }

    pub fn store() {
        GLOBAL_STATE.0.borrow_mut().store_into_kvpair();
    }

    pub fn initialize() {
        GLOBAL_STATE.0.borrow_mut().fetch();
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
const CANCELL_LOTTERY: u64 = 7;
const WITHDRAW: u64 = 8;

const ERROR_PLAYER_ALREADY_EXIST:u32 = 1;
const ERROR_PLAYER_NOT_EXIST:u32 = 2;
const ERROR_NOT_SELECTED_PLAYER:u32 = 3;
const SELECTED_PLAYER_NOT_EXIST: u32 = 4;
const PLAYER_ACTION_NOT_FINISHED: u32 = 5;
const PLAYER_LOTTERY_EXPIRED: u32 = 6;
const PLAYER_LOTTERY_PROGRESS_NOT_FULL: u32 = 7;

pub struct Transaction {
    command: u64,
    nonce: u64,
    data: Vec<u64>
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
        let mut data = vec![];
        if command == WITHDRAW {
            data = vec![params[1], params[2], params[3]] // address of withdraw(Note:amount in params[1])
        }

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
                let player = Player::new(pkey);
                player.store();
                let mut state = GLOBAL_STATE.0.borrow_mut();

                state.player_list.push(QueryPlayerState {
                    pid: [player.player_id[0], player.player_id[1]],
                    data: player.data.clone()
                });
                0
            }
        }
    }

    pub fn action(&self, pkey: &[u64; 4], action: u64, rand: &[u64; 4]) -> u32 {
        let mut player = PuppyPlayer::get(pkey);
        let counter = {
            let state = GLOBAL_STATE.0.borrow();
            state.counter
        };
        match player.as_mut() {
            None => ERROR_PLAYER_NOT_EXIST,
            Some(player) => {
                // Check for Lottery action
                if action == LOTTERY {
                    // This is the selected player; allow them to open the blind box
                    zkwasm_rust_sdk::dbg!("Player {:?} is opening the blind box", pkey);
                    if player.data.progress == 1000 {
                        if player.data.last_lottery_timestamp + 10 > counter {
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
                            0 // return 0 here instead of return error
                            // PLAYER_LOTTERY_EXPIRED
                        }
                    } else {
                        PLAYER_LOTTERY_PROGRESS_NOT_FULL
                    }
                } else if action == CANCELL_LOTTERY {
                    player.data.action = SWAY;
                    player.data.progress = 0;
                    player.data.last_lottery_timestamp = 0;
                    player.store();
                    0
                } else if action == WITHDRAW {
                    let mut player = PuppyPlayer::get(pkey);
                    match player.as_mut() {
                        None => ERROR_PLAYER_NOT_EXIST,
                        Some(player) => {
                            player.check_and_inc_nonce(self.nonce);
                            let balance = player.data.balance;
                            let amount = self.data[0] & 0xffffffff;
                            unsafe { require(balance >= amount) };
                            player.data.balance -= amount;
                            zkwasm_rust_sdk::dbg!("balance: {}, amount is {:?}", balance, amount);
                            let withdrawinfo = WithdrawInfo::new(&[
                                self.data[0],
                                self.data[1],
                                self.data[2]
                            ]);
                            SettlementInfo::append_settlement(withdrawinfo);
                            player.store();
                            0
                        }
                    }
                } else {
                    let action_duration = get_action_duration();
                    let action_reward = get_action_reward();

                    if player.data.last_action_timestamp != 0
                        && counter < player.data.last_action_timestamp + action_duration {
                        PLAYER_ACTION_NOT_FINISHED
                    } else {
                        player.data.action = action;
                        player.data.last_action_timestamp = counter;
                        player.data.progress += action_reward;
                        if player.data.progress == 1000 {
                            player.data.last_lottery_timestamp = counter;
                        } else if player.data.progress > 1000 {
                            player.data.progress = 1000;
                        }
                        GlobalState::update_player_list(player);
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
            CANCELL_LOTTERY => self.action(pkey, CANCELL_LOTTERY, rand),
            WITHDRAW => self.action(pkey, WITHDRAW, rand),
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
