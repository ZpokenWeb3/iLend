use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Timestamp;
use cosmwasm_std::Uint128;

use pyth_sdk_cw::{Price, PriceIdentifier};

#[cw_serde]
pub struct InstantiateMsg {
    // accounts that are allowed to initiate deposits and withdrawals
    pub lending_contract: String,
    pub margin_contract: String,
    pub admin: String,
}

#[cw_serde]
pub enum ExecuteMsg {
    SetLendingContract { contract: String },
    SetMarginContract { contract: String },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(String)]
    GetLendingContract,

    #[returns(String)]
    GetMarginContract,
}

