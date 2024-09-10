use wasm_bindgen::prelude::*;
use zkwasm_rest_abi::*;
pub mod config;
pub mod player;
pub mod state;
pub mod settlement;
//pub mod reward;

use crate::config::Config;
use crate::state::{GlobalState, Transaction};
zkwasm_rest_abi::create_zkwasm_apis!(Transaction, GlobalState, Config);
