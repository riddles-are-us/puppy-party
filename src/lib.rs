use wasm_bindgen::prelude::*;
use zkwasm_rest_abi::*;
pub mod config;
pub mod events;
pub mod player;
pub mod state;
pub mod settlement;

use crate::config::Config;
use crate::state::{GlobalState, Transaction};
zkwasm_rest_abi::create_zkwasm_apis!(Transaction, GlobalState, Config);