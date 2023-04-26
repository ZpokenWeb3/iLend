use crate::msg::{RepayInfo, TokenInfo, TokenInterestRateModelParams};
use {
    cosmwasm_std::Uint128,
    cw_storage_plus::{Item, Map},
};

pub const USER_DEPOSITED_BALANCE: Map<(String, String), Uint128> =
    Map::new("user_deposited_balance");
/*
USER_DEPOSITED_BALANCE STORAGE
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

pub const USER_REPAY_INFO: Map<(String, String), RepayInfo> = Map::new("user_repay_info");
/*
USER_REPAY_INFO STORAGE
Key: (user_address_1, token_A) -> Value: repay_info
Key: (user_address_1, token_B) -> Value: repay_info
Key: (user_address_2, token_A) -> Value: repay_info
 */

pub const TOKENS_INTEREST_RATE_MODEL_PARAMS: Map<String, TokenInterestRateModelParams> = Map::new("token_interest_rate_model_params");
/*
TOKENS_INTEREST_RATE_MODEL_PARAMS STORAGE
Key: token demon -> Value: TokenInterestRateModelParams
*/
