use crate::msg::TokenInfo;
use cosmwasm_std::Addr;
use pyth_sdk_cw::PriceIdentifier;
use {
    cosmwasm_std::Uint128,
    cw_storage_plus::{Item, Map},
};

// accounts that are responsible for storing collateral to initiate deposits and withdrawals
pub const COLLATERAL_VAULT: Item<String> = Item::new("collateral_vault_margin");
pub const LENDING_CONTRACT: Item<String> = Item::new("lending_margin");

// contract itself, or other contract that are allowed to alter sensitive information
pub const ADMIN: Item<String> = Item::new("admin_margin");

pub const PRICE_FEED_IDS: Map<String, PriceIdentifier> = Map::new("price_feed_ids_margin");
pub const PYTH_CONTRACT: Item<Addr> = Item::new("pyth_contract");

pub const IS_TESTING: Item<bool> = Item::new("is_testing_margin");

pub const PRICES: Map<String, u128> = Map::new("prices_margin");

pub const PRICE_UPDATER_CONTRACT: Item<String> = Item::new("price_updater_margin");

pub const SUPPORTED_TOKENS: Map<String, TokenInfo> = Map::new("tokens_margin");

pub const USER_MM_TOKEN_BALANCE: Map<(String, String), Uint128> =
    Map::new("user_mm_token_balance_margin");
/*
USER_MM_TOKEN_BALANCE STORAGE
Key: (user_address_1, token_A) -> Value: balance_for_token_A
Key: (user_address_1, token_B) -> Value: balance_for_token_B
Key: (user_address_2, token_A) -> Value: balance_for_token_A
 */
