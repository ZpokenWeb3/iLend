use cosmwasm_std::Uint128;

use cosmwasm_schema::{cw_serde, QueryResponses};

// cw_serde attribute is equivalent to
// #[derive(Serialize, Deserialize, PartialEq, Debug, Clone, JsonSchema)]
// #[serde(rename_all = "snake_case")]

#[cw_serde]
pub struct InstantiateMsg {
    pub admin: String,
    // name, denom, symbol, decimals
    pub supported_tokens: Vec<(String, String, String, u128)>,
}

#[cw_serde]
pub enum ExecuteMsg {
    // Admin-only functionality for funding contract with reserves
    // to be able to operate borrows and repayments
    Fund {},
    SetPrice {
        denom: String,
        price: u128,
    },
    AddMarkets {
        denom: String,
        name: String,
        symbol: String,
        decimals: u128,
    },

    // Deposit / Redeem functionality
    Deposit {},
    Redeem {
        denom: String,
        amount: Uint128,
    },

    // Borrow / Repay functionality
    Borrow {
        denom: String,
        amount: Uint128,
    },
    Repay {},
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(GetBalanceResponse)]
    GetDeposit { address: String, denom: String },

    #[returns(GetBorrowsResponse)]
    GetBorrows { address: String, denom: String },

    #[returns(RepayInfo)]
    GetRepayInfo { address: String, denom: String },

    #[returns(GetSupportedTokensResponse)]
    GetSupportedTokens {},

    #[returns(GetPriceResponse)]
    GetPrice { denom: String },

    #[returns(GetUserDepositedUsdResponse)]
    GetUserDepositedUsd { address: String },

    #[returns(GetUserBorrowedUsdResponse)]
    GetUserBorrowedUsd { address: String },

    #[returns(Uint128)]
    GetContractBalance { denom: String },

    #[returns(Uint128)]
    GetAvailableToBorrow { address: String, denom: String },

    #[returns(Uint128)]
    GetAvailableToRedeem { address: String, denom: String },

    #[returns(Uint128)]
    GetTotalReservesByToken { denom: String },

    #[returns(Uint128)]
    GetTotalDepositedByToken { denom: String },

    #[returns(Uint128)]
    GetTotalBorrowedByToken { denom: String },

    #[returns(Uint128)]
    GetAvailableLiquidityByToken { denom: String },

    #[returns(Uint128)]
    GetUtilizationRateByToken { denom: String },
}


#[cw_serde]
pub struct GetPriceResponse {
    pub price: u128,
}

#[cw_serde]
pub struct GetBalanceResponse {
    pub balance: Uint128,
}

#[cw_serde]
pub struct GetBorrowsResponse {
    pub borrows: Uint128,
}

#[cw_serde]
pub struct GetUserDepositedUsdResponse {
    pub user_deposited_usd: Uint128,
}

#[cw_serde]
pub struct GetUserBorrowedUsdResponse {
    pub user_borrowed_usd: Uint128,
}

#[cw_serde]
pub struct GetSupportedTokensResponse {
    pub supported_tokens: Vec<TokenInfo>,
}

#[cw_serde]
pub struct RepayInfo {
    pub borrowed_amount: Uint128,
    pub accumulated_interest: Uint128,
}

impl Default for RepayInfo {
    fn default() -> Self {
        RepayInfo {
            borrowed_amount: Default::default(),
            accumulated_interest: Default::default(),
        }
    }
}

#[cw_serde]
pub struct GetTotalDepositedUsdResponse {
    pub total_deposited_usd: Uint128,
}

#[cw_serde]
pub struct GetTotalBorrowedUsdResponse {
    pub total_borrowed_usd: Uint128,
}

#[cw_serde]
pub struct TokenInfo {
    pub denom: String,
    pub name: String,
    pub symbol: String,
    pub decimals: u128,
}
