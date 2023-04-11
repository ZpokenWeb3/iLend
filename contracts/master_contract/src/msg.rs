use cosmwasm_schema::cw_serde;
use cosmwasm_std::Uint128;

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

    // Deposit / Withdraw functionality for users
    Deposit {},
    Withdraw { denom: String, amount: Uint128 },
}

#[cw_serde]
pub enum QueryMsg {
    GetDeposit { address: String, denom: String },
}
