use {
    cosmwasm_std::Uint128,
    cw_storage_plus::{Item, Map},
};

pub const USER_PROFILES: Map<(String, String), Uint128> = Map::new("user_profiles");
/*
USER PROFILE STORAGE
Key: (user_address_1, token_A) -> Value: balance_for_token_A
Key: (user_address_1, token_B) -> Value: balance_for_token_B
Key: (user_address_2, token_A) -> Value: balance_for_token_A
 */

pub const SUPPORTED_TOKENS: Map<String, String> = Map::new("tokens");
/*
SUPPORTED_TOKENS STORAGE
Key: token demon -> Value: itoken denom
*/

pub const ADMIN: Item<String> = Item::new("admin");
/*
ADMIN that are eligible to fund contract with reserves [ contract itself by default ]
*/
