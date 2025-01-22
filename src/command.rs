use crate::config::{get_action_duration, get_action_reward};
use zkwasm_rust_sdk::require;
use zkwasm_rest_abi::WithdrawInfo;
use crate::settlement::SettlementInfo;
use crate::player::PuppyPlayer;
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
    Tick,
}


pub trait CommandHandler {
    fn handle(&self, pid: &[u64; 2], nonce: u64, rand: &[u64; 4]) -> Result<(), u32>;
}

#[derive (Clone)]
pub struct Withdraw {
    pub data: [u64; 3],
}

impl CommandHandler for Withdraw {
    fn handle(&self, pid: &[u64; 2], nonce: u64, _rand: &[u64; 4]) -> Result<(), u32> {
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
    fn handle(&self, pid: &[u64; 2], nonce: u64, _rand: &[u64; 4]) -> Result<(), u32> {
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
    fn handle(&self, pid: &[u64; 2], nonce: u64, _rand: &[u64; 4]) -> Result<(), u32> {
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
    Stake(usize),
    Bet(usize),
    Comment(Vec<u8>),
    Lottery,
}

impl CommandHandler for Activity {
    fn handle(&self, pid: &[u64; 2], nonce: u64, rand: &[u64; 4]) -> Result<(), u32> {
        let counter = GlobalState::get_counter();
        let mut player = PuppyPlayer::get_from_pid(pid);
        match player.as_mut() {
            None => Err(ERROR_PLAYER_NOT_EXIST),
            Some(player) => {
                match self {
                    Activity::Stake(sz) => {
                        let amount = sz;
                        player.check_and_inc_nonce(nonce);
                        player.data.cost_ticket(amount)?;
                        player.data.stake[sz] += amount;
                        GlobalState::update_meme_stake(*sz, player);
                        player.store();
                        Ok(())
                    },
                    Activity::Vote(sz) | Activity::Bet(sz) => {
                        let action_duration = get_action_duration();
                        player.data.check_and_update_action_timestamp(counter, action_duration)?;
                        let action_reward = get_action_reward();
                        player.data.cost_ticket(1)?;
                        player.data.increase_progress(counter,action_reward);
                        player.check_and_inc_nonce(nonce);
                        GlobalState::update_meme_rank(*sz);
                        player.store();
                        Ok(())
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
        _ => "Unknown",
    }
}
