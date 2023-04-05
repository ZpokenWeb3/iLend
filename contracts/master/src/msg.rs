use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Uint128;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[cw_serde]
pub struct InstantiateMsg {}

#[cw_serde]
pub enum ExecuteMsg {
    Transfer { recipient: String, amount: Uint128 },
}


#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    /// Returns the current balance of the given address, 0 if unset.
    #[returns(cw20::BalanceResponse)]
    Balance { address: String },
}

