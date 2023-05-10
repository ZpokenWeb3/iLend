use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Timestamp;
use cosmwasm_std::Uint128;

// cw_serde attribute is equivalent to
// #[derive(Serialize, Deserialize, PartialEq, Debug, Clone, JsonSchema)]
// #[serde(rename_all = "snake_case")]

#[cw_serde]
pub struct InstantiateMsg {
    pub admin: String,
    // name, denom, symbol, decimals
    pub supported_tokens: Vec<(String, String, String, u128)>,
    // denom, min_interest_rate, safe_borrow_max_rate, rate_growth_factor
    pub tokens_interest_rate_model_params: Vec<(String, u128, u128, u128)>,
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
        min_interest_rate: u128,
        safe_borrow_max_rate: u128,
        rate_growth_factor: u128,
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
    ToggleCollateralSetting {
        denom: String,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(GetBalanceResponse)]
    GetDeposit { address: String, denom: String },

    #[returns(bool)]
    UserDepositAsCollateral { address: String, denom: String },

    #[returns(GetBorrowAmountWithInterestResponse)]
    GetBorrowAmountWithInterest { address: String, denom: String },

    #[returns(UserBorrowingInfo)]
    GetUserBorrowingInfo { address: String, denom: String },

    #[returns(TotalBorrowData)]
    GetTotalBorrowData { denom: String },

    #[returns(GetSupportedTokensResponse)]
    GetSupportedTokens {},

    #[returns(GetTokensInterestRateModelParamsResponse)]
    GetTokensInterestRateModelParams {},

    #[returns(Uint128)]
    GetPrice { denom: String },

    #[returns(Uint128)]
    GetInterestRate { denom: String },

    #[returns(Uint128)]
    GetLiquidityRate { denom: String },

    #[returns(Uint128)]
    GetCurrentLiquidityIndexLn { denom: String },

    #[returns(Uint128)]
    GetMmTokenPrice { denom: String },

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

    #[returns(Uint128)]
    GetLiquidityIndexLastUpdate { denom: String },
}

#[cw_serde]
pub struct GetBalanceResponse {
    pub balance: Uint128,
}

#[cw_serde]
pub struct GetBorrowAmountWithInterestResponse {
    pub amount: Uint128,
    pub base: Uint128,
    pub exponent: Uint128,
    pub average_interest_rate: Uint128,
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
pub struct GetTokensInterestRateModelParamsResponse {
    pub tokens_interest_rate_model_params: Vec<TokenInterestRateModelParams>,
}

#[cw_serde]
pub struct UserBorrowingInfo {
    pub borrowed_amount: Uint128,
    pub average_interest_rate: Uint128,
    pub timestamp: Timestamp,
}

impl Default for UserBorrowingInfo {
    fn default() -> Self {
        UserBorrowingInfo {
            borrowed_amount: Uint128::zero(),
            average_interest_rate: Uint128::zero(),
            timestamp: Default::default(),
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

#[cw_serde]
pub struct TokenInterestRateModelParams {
    pub denom: String,
    pub min_interest_rate: u128,
    pub safe_borrow_max_rate: u128,
    pub rate_growth_factor: u128,
}

#[cw_serde]
pub struct LiquidityIndexData {
    pub denom: String,
    pub liquidity_index_ln: u128,
    pub timestamp: Timestamp,
}

#[cw_serde]
#[derive(Default)]
pub struct TotalBorrowData {
    pub denom: String,
    pub total_borrowed_amount: u128,
    pub expected_annual_interest_income: u128,
}
