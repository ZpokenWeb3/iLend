use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Uint128;
use cosmwasm_std::{Addr, Timestamp};
use cw20::Cw20ReceiveMsg;

use pyth_sdk_cw::{Price, PriceIdentifier};

#[cw_serde]
pub struct InstantiateMsg {
    // different sources for testing and production
    pub is_testing: bool,
    pub admin: String,
    // name, denom, symbol, decimals
    pub supported_tokens: Vec<(String, String, String, Option<String>, u128)>,
    // denom, loan_to_value_ratio, liquidation_threshold
    pub reserve_configuration: Vec<(String, u128, u128)>,
    // denom, min_interest_rate, safe_borrow_max_rate, rate_growth_factor, optimal_utilisation_ratio
    pub tokens_interest_rate_model_params: Vec<(String, u128, u128, u128, u128)>,
    // vector of (token denom, price_identifier) got from https://pyth.network/developers/price-feed-ids#cosmwasm-testnet
    pub price_ids: Vec<(String, PriceIdentifier)>,
    // pyth contract on a given network that fetches prices | testnet & mainnet
    pub pyth_contract_addr: String,
    // updater service that is eligible to update price
    pub price_updater_addr: String,
}

#[cw_serde]
pub enum ExecuteMsg {
    // Receive hook for Cw20 Send messages
    // for depositing Token Factory Token
    Receive(Cw20ReceiveMsg),

    // for depositing ERC20 Tokens, IBC Token and INJ
    Deposit {},
    Redeem {
        denom: String,
        amount: Uint128,
    },
    Borrow {
        denom: String,
        amount: Uint128,
    },
    Repay {},
    Liquidation {
        user: String,
    },
    UpdatePrice {
        denom: Option<String>,
        price: Option<u128>,
    },
    SetReserveConfiguration {
        denom: String,
        loan_to_value_ratio: u128,
        liquidation_threshold: u128,
    },
    SetTokenInterestRateModelParams {
        denom: String,
        min_interest_rate: u128,
        safe_borrow_max_rate: u128,
        rate_growth_factor: u128,
        optimal_utilisation_ratio: u128,
    },
    AddMarkets {
        denom: String,
        name: String,
        symbol: String,
        decimals: u128,
        cw20_address: Option<String>,
        loan_to_value_ratio: u128,
        liquidation_threshold: u128,
        min_interest_rate: u128,
        safe_borrow_max_rate: u128,
        rate_growth_factor: u128,
        optimal_utilisation_ratio: u128,
    },

    ToggleCollateralSetting {
        denom: String,
    },

    UpdatePythContract {
        pyth_contract_addr: String,
    },
    UpdatePriceUpdaterAddr {
        price_updater_addr: String,
    },
    AddPriceFeedIds {
        price_ids: Vec<(String, PriceIdentifier)>,
    },
    UpdateAdmin {
        admin: String,
    },
}

#[cw_serde]
pub enum Cw20HookMsg {
    Deposit { denom: String },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(GetBalanceResponse)]
    GetDeposit { address: String, denom: String },

    #[returns(bool)]
    UserDepositAsCollateral { address: String, denom: String },

    #[returns(Uint128)]
    GetUserBorrowAmountWithInterest { address: String, denom: String },

    #[returns(UserBorrowingInfo)]
    GetUserBorrowingInfo { address: String, denom: String },

    #[returns(TotalBorrowData)]
    GetTotalBorrowData { denom: String },

    #[returns(GetSupportedTokensResponse)]
    GetSupportedTokens {},

    #[returns(GetReserveConfigurationResponse)]
    GetReserveConfiguration {},

    #[returns(GetTokensInterestRateModelParamsResponse)]
    GetTokensInterestRateModelParams {},

    #[returns(Uint128)]
    GetPrice { denom: String },

    #[returns(String)]
    GetPythContract {},

    #[returns(Uint128)]
    GetInterestRate { denom: String },

    #[returns(Uint128)]
    GetLiquidityRate { denom: String },

    #[returns(Uint128)]
    GetCurrentLiquidityIndexLn { denom: String },

    #[returns(Uint128)]
    GetMmTokenPrice { denom: String },

    #[returns(Uint128)]
    GetUserDepositedUsd { address: String },

    #[returns(Uint128)]
    GetUserCollateralUsd { address: String },

    #[returns(Uint128)]
    GetUserBorrowedUsd { address: String },

    #[returns(Uint128)]
    GetUserUtilizationRate { address: String },

    #[returns(Uint128)]
    GetUserLiquidationThreshold { address: String },

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

    #[returns(Uint128)]
    GetUserMaxAllowedBorrowAmountUsd { address: String },

    #[returns(Vec < String >)]
    GetAllUsersWithBorrows {},

    #[returns(Vec < (String, PriceIdentifier) >)]
    GetPriceFeedIds {},

    #[returns(String)]
    GetAdmin {},

    #[returns(Vec < (String, Uint128) >)]
    GetUserBalances { address: String },
}

#[cw_serde]
pub struct GetBalanceResponse {
    pub balance: Uint128,
}

#[cw_serde]
pub struct GetSupportedTokensResponse {
    pub supported_tokens: Vec<TokenInfo>,
}

#[cw_serde]
pub struct GetReserveConfigurationResponse {
    pub reserve_configuration: Vec<ReserveConfiguration>,
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

#[cw_serde]
pub struct UserDataByToken {
    pub deposited: Uint128,
    pub borrowed: Uint128,
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
    pub cw20_address: Option<String>,
}

#[cw_serde]
pub struct ReserveConfiguration {
    pub denom: String,
    pub loan_to_value_ratio: u128,
    // LTV ratio
    pub liquidation_threshold: u128,
}

#[cw_serde]
pub struct TokenInterestRateModelParams {
    pub denom: String,
    pub min_interest_rate: u128,
    pub safe_borrow_max_rate: u128,
    pub rate_growth_factor: u128,
    pub optimal_utilisation_ratio: u128,
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
    pub average_interest_rate: u128,
    pub timestamp: Timestamp,
}
