use crate::config::{get_action_duration, get_action_reward};
use crate::meme::{MemeInfo, StakeInfo};
use zkwasm_rest_convention::{IndexedObject, Position};
use zkwasm_rust_sdk::require;
use zkwasm_rest_abi::WithdrawInfo;
use crate::settlement::SettlementInfo;
use crate::player::{PositionHolder, PuppyPlayer};
use crate::state::GlobalState;
use crate::error::*;

#[derive (Clone)]
pub enum Command {
    // standard activities
    Activity(Activity),
    // standard withdraw and deposit
    Withdraw(Withdraw),
    WithdrawLottery(WithdrawLottery),
    Deposit(Deposit),
    // standard player install and timer
    InstallPlayer,
    InstallMeme(u64),
    Tick,
}


pub trait CommandHandler {
    fn handle(&self, pid: &[u64; 2], nonce: u64, rand: &[u64; 4], _counter: u64) -> Result<(), u32>;
}

#[derive (Clone)]
pub struct Withdraw {
    pub data: [u64; 3],
}

impl CommandHandler for Withdraw {
    fn handle(&self, pid: &[u64; 2], nonce: u64, _rand: &[u64; 4], _counter: u64) -> Result<(), u32> {
        let mut player = PuppyPlayer::get_from_pid(pid);
        match player.as_mut() {
            None => Err(ERROR_PLAYER_NOT_EXIST),
            Some(player) => {
                player.check_and_inc_nonce(nonce);
                let balance = player.data.balance;
                let amount = (self.data[0] & 0xffffffff) as u32;
                unsafe { require(balance >= amount) };
                player.data.balance -= amount;
                let withdrawinfo =
                    WithdrawInfo::new(&[self.data[0], self.data[1], self.data[2]], 0);
                SettlementInfo::append_settlement(withdrawinfo);
                player.store();
                Ok(())
            }
        }
    }
}

#[derive (Clone)]
pub struct WithdrawLottery {
    pub data: [u64; 3],
}

impl CommandHandler for WithdrawLottery {
    fn handle(&self, pid: &[u64; 2], nonce: u64, _rand: &[u64; 4], _counter: u64) -> Result<(), u32> {
        let mut player = PuppyPlayer::get_from_pid(pid);
        match player.as_mut() {
            None => Err(ERROR_PLAYER_NOT_EXIST),
            Some(player) => {
                player.check_and_inc_nonce(nonce);
                let balance = player.data.lottery_info;
                let amount = (self.data[0] & 0xffffffff) as u32;
                unsafe { require(balance >= amount) };
                player.data.lottery_info -= amount;
                let withdrawinfo =
                    WithdrawInfo::new(&[self.data[0], self.data[1], self.data[2]], 1<<8);
                SettlementInfo::append_settlement(withdrawinfo);
                player.store();
                Ok(())
            }
        }
    }
}

#[derive (Clone)]
pub struct Deposit {
    pub data: [u64; 3],
}

impl CommandHandler for Deposit {
    fn handle(&self, pid: &[u64; 2], nonce: u64, _rand: &[u64; 4], _counter: u64) -> Result<(), u32> {
        let mut admin = PuppyPlayer::get_from_pid(pid).unwrap();
        admin.check_and_inc_nonce(nonce);
        let mut player = PuppyPlayer::get_from_pid(&[self.data[0], self.data[1]]);
        match player.as_mut() {
            None => Err(ERROR_PLAYER_NOT_EXIST),
            Some(player) => {
                player.data.ticket += self.data[2] as u32;
                player.store();
                admin.store();
                Ok(())
            }
        }
    }
}

#[derive (Clone)]
pub enum Activity {
    // activities
    Vote(usize),
    Stake(usize, u64),
    Collect(usize),
    Comment(Vec<u8>),
    Lottery,
}

impl CommandHandler for Activity {
    fn handle(&self, pid: &[u64; 2], nonce: u64, rand: &[u64; 4], counter: u64) -> Result<(), u32> {
        let counter = GlobalState::get_counter();
        let mut player = PuppyPlayer::get_from_pid(pid);
        match player.as_mut() {
            None => Err(ERROR_PLAYER_NOT_EXIST),
            Some(player) => {
                match self {
                    Activity::Stake(sz, amount) => {
                        player.check_and_inc_nonce(nonce);
                        let meme_id = *sz as u64;
                        let (pos, meme) = player.stake(meme_id, *amount as u32, counter)?;
                        player.store();
                        meme.store();
                        pos.store();
                        StakeInfo::emit_event(&pid, meme_id, &pos.data);
                        MemeInfo::emit_event(meme_id, &meme.data);
                        Ok(())
                    },
                    Activity::Collect(sz) => {
                        player.check_and_inc_nonce(nonce);
                        let meme_id = *sz as u64;
                        let pos = player.collect(meme_id, counter)?;
                        player.store();
                        pos.store();
                        StakeInfo::emit_event(&pid, meme_id, &pos.data);
                        Ok(())
                    },
                    Activity::Vote(sz) => {
                        let action_duration = get_action_duration();
                        player.data.check_and_update_action_timestamp(counter, action_duration)?;
                        let action_reward = get_action_reward();
                        player.data.cost_ticket(1)?;
                        player.data.increase_progress(counter,action_reward);
                        player.check_and_inc_nonce(nonce);
                        let meme_id = *sz as u64;
                        let meme = MemeInfo::get_object(meme_id);
                        match meme {
                            None => Err(INVALID_MEME_INDEX),
                            Some (mut m) => {
                                m.data.rank += 1;
                                m.store();
                                player.store();
                                MemeInfo::emit_event(meme_id, &m.data);
                                Ok(())
                            }
                        }
                    },
                    Activity::Lottery => {
                        // This is the selected player; allow them to open the blind box
                        if player.data.progress == 1000 {
                            player.check_and_inc_nonce(nonce);
                            player.data.action = 0;
                            player.data.progress = 0;
                            player.data.last_lottery_timestamp = 0;
                            player.data.last_action_timestamp = 0;

                            // set lottery_info if the last 16 bit are 1
                            if (rand[1] & 0xff) > 0xf0 {
                                //zkwasm_rust_sdk::dbg!("rand is {}", {rand[1]});
                                player.data.lottery_info += 10; // change 10 to random reward
                            } else {
                                player.data.balance += 10; // change 10 to random reward
                            }
                            player.store();
                            Ok(())
                        } else {
                            Err(PLAYER_LOTTERY_PROGRESS_NOT_FULL)
                        }
                    },
                    Activity::Comment(_) => {
                        unreachable!()
                    }
                }
            }
        }
    }
}

pub fn decode_error(e: u32) -> &'static str {
    match e {
        ERROR_PLAYER_NOT_EXIST => "PlayerNotExist",
        ERROR_PLAYER_ALREADY_EXIST => "PlayerAlreadyExist",
        ERROR_NOT_SELECTED_PLAYER => "PlayerNotSelected",
        SELECTED_PLAYER_NOT_EXIST => "SelectedPlayerNotExist",
        PLAYER_ACTION_NOT_FINISHED => "PlayerActionNotFinished",
        PLAYER_LOTTERY_EXPIRED => "PlayerLotteryExpired",
        PLAYER_LOTTERY_PROGRESS_NOT_FULL => "PlayerLotteryProgressNotFull",
        PLAYER_NOT_ENOUGH_TICKET => "PlayerNotEnoughTicket",
        INVALID_MEME_INDEX => "SpecifiedMemeIndexNotFound",
        _ => "Unknown",
    }
}
