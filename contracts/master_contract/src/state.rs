use crate::msg::{
    LiquidityIndexData, TokenInfo, ReserveConfiguration, TokenInterestRateModelParams, TotalBorrowData, UserBorrowingInfo,
};
use cosmwasm_std::Addr;
use pyth_sdk_cw::PriceIdentifier;
use std::ops::Add;
use {
    cosmwasm_std::Uint128,
    cw_storage_plus::{Item, Map},
};

pub const USER_MM_TOKEN_BALANCE: Map<(String, String), Uint128> = Map::new("user_mm_token_balance");
/*
USER_MM_TOKEN_BALANCE STORAGE
Key: (user_address_1, token_A) -> Value: balance_for_token_A
Key: (user_address_1, token_B) -> Value: balance_for_token_B
Key: (user_address_2, token_A) -> Value: balance_for_token_A
 */

pub const USER_DEPOSIT_AS_COLLATERAL: Map<(String, String), bool> =
    Map::new("user_deposit_as_collateral");
/*
USER_DEPOSIT_AS_COLLATERAL STORAGE
Key: (user_address_1, token_A) -> Value: user_deposit_as_collateral
Key: (user_address_1, token_B) -> Value: user_deposit_as_collateral
Key: (user_address_2, token_A) -> Value: user_deposit_as_collateral
 */

pub const PRICES: Map<String, u128> = Map::new("prices");

pub const SUPPORTED_TOKENS: Map<String, TokenInfo> = Map::new("tokens");
/*
SUPPORTED_TOKENS STORAGE
Key: token demon -> Value: TokenInfo
*/

pub const ADMIN: Item<String> = Item::new("admin");
/*
ADMIN that are eligible to fund contract with reserves [ contract itself by default ]
*/

pub const USER_BORROWING_INFO: Map<(String, String), UserBorrowingInfo> =
    Map::new("user_borrowing_info");
/*
USER_BORROWING_INFO STORAGE
Key: (user_address_1, token_A) -> Value: user_borrowing_info
Key: (user_address_1, token_B) -> Value: user_borrowing_info
Key: (user_address_2, token_A) -> Value: user_borrowing_info
 */

pub const RESERVE_CONFIGURATION: Map<String, ReserveConfiguration> =
    Map::new("reserve_configuration");
/*
RESERVE_CONFIGURATION STORAGE
Key: token demon -> Value: ReserveConfiguration
*/

pub const TOKENS_INTEREST_RATE_MODEL_PARAMS: Map<String, TokenInterestRateModelParams> =
    Map::new("token_interest_rate_model_params");
/*
TOKENS_INTEREST_RATE_MODEL_PARAMS STORAGE
Key: token demon -> Value: TokenInterestRateModelParams
*/

pub const LIQUIDITY_INDEX_DATA: Map<String, LiquidityIndexData> = Map::new("liquidity_index_data");
/*
LIQUIDITY_INDEX_DATA STORAGE
Key: token demon -> Value: LiquidityIndexData
*/

pub const TOTAL_BORROW_DATA: Map<String, TotalBorrowData> = Map::new("total_borrow_data");
/*
TOTAL_BORROW_DATA STORAGE
Key: token demon -> Value: TotalBorrowData
*/

// mapping of (token denom, price_identifier)
pub const PRICE_FEED_IDS: Map<String, PriceIdentifier> = Map::new("price_feed_ids");

// Contract address of Pyth on  Injective testnet
pub const PYTH_CONTRACT: Item<Addr> = Item::new("pyth_contract");

pub const IS_TESTING: Item<bool> = Item::new("is_testing");
