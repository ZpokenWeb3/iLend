use pyth_sdk_cw::{Price, PriceIdentifier};
use {
    cosmwasm_std::Uint128,
};

use cosmwasm_schema::{
    cw_serde,
    QueryResponses,
};

// cw_serde attribute is equivalent to
// #[derive(Serialize, Deserialize, PartialEq, Debug, Clone, JsonSchema)]
// #[serde(rename_all = "snake_case")]


#[cw_serde]
pub struct InstantiateMsg {
    pub admin: String,
    pub supported_tokens: Vec<(String, String)>,
}

#[cw_serde]
pub enum ExecuteMsg {
    // Admin-only functionality for funding contract with reserves
    // to be able to operate borrows and repayments
    Fund {},
    AddMarkets { token: String, itoken: String },

    // Deposit / Withdraw functionality for users
    Deposit {},
    Redeem { denom: String, amount: Uint128 },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(GetBalanceResponse)]
    GetDeposit { address: String, denom: String },
}

#[cw_serde]
pub struct GetBalanceResponse {
    pub balance: Uint128,
}