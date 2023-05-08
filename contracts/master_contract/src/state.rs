use crate::msg::{
    LiquidityIndexData, TokenInfo, TokenInterestRateModelParams, TotalBorrowData, UserBorrowingInfo,
};
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

pub const USER_BORROWED_BALANCE: Map<(String, String), Uint128> = Map::new("user_borrowed_balance");
/*
USER_BORROWED_BALANCE STORAGE
Key: (user_address_1, token_A) -> Value: borrowed_amount_of_token_A
Key: (user_address_1, token_B) -> Value: borrowed_amount_of_token_B
Key: (user_address_2, token_A) -> Value: borrowed_amount_of_token_A
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
