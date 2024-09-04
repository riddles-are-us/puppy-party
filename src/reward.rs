use crate::player::{PuppyPlayer, Owner};

#[derive(Debug, Clone)]
pub enum DrinkReward {
    Champagne,
    Cognac,
    Whiskey,
    Wine,
    Tequila,
    Vodka,
    Cocktail,
    Beer
}

impl DrinkReward {
    pub fn reward_value(&self) -> u64 {
        match self {
            DrinkReward::Champagne => 100,
            DrinkReward::Cognac => 80,
            DrinkReward::Whiskey => 60,
            DrinkReward::Wine => 50,
            DrinkReward::Tequila => 40,
            DrinkReward::Vodka => 30,
            DrinkReward::Cocktail => 70,
            DrinkReward::Beer => 10
        }
    }
}

fn select_random_drink_reward() -> DrinkReward {
    let mut rng = urandom::new();
    let reward = match rng.range(0..=100) {
        0..=10 => DrinkReward::Champagne,   // 10% chance
        11..=20 => DrinkReward::Cognac,     // 10% chance
        21..=40 => DrinkReward::Whiskey,    // 20% chance
        41..=60 => DrinkReward::Wine,       // 20% chance
        61..=70 => DrinkReward::Tequila,    // 10% chance
        71..=80 => DrinkReward::Vodka,      // 10% chance
        81..=100 => DrinkReward::Beer,      // 20% chance
        _ => DrinkReward::Cocktail,         // Default to Cocktail (unlikely)
    };

    reward
}

pub fn assign_reward_to_player(player: &mut PuppyPlayer) {
    let reward = select_random_drink_reward();
    player.set_reward(reward.reward_value());
    let player_id = player.player_id;
    zkwasm_rust_sdk::dbg!("Player {:?} has received a reward", player_id);
}