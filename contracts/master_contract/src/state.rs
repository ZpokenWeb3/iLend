use cosmwasm_std::{Uint128};
use cw_storage_plus::{Item, Map};

/*
USER PROFILE STORAGE
Key: (user_address_1, token_A) -> Value: balance_for_token_A
Key: (user_address_1, token_B) -> Value: balance_for_token_B
Key: (user_address_2, token_A) -> Value: balance_for_token_A
 */

pub const USER_PROFILES: Map<(String, String), Uint128> = Map::new("user_profiles");
pub const VAULT_CONTRACT: Item<String> = Item::new("vault_contract");