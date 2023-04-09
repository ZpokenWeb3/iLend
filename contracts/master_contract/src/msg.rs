
use cosmwasm_schema::{cw_serde, QueryResponses};


#[cw_serde]
pub struct InstantiateMsg {
    pub vault: String,
    pub denom: String,
}

#[cw_serde]
pub enum ExecuteMsg {
    Deposit {},
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {}