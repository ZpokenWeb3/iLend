use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Uint128;

#[cw_serde]
pub struct InstantiateMsg {
    // accounts that are allowed to initiate deposits and withdrawals
    pub lending_contract: String,
    pub margin_contract: String,
    pub admin: String,
}

#[cw_serde]
pub enum ExecuteMsg {
    SetLendingContract {
        contract: String,
    },
    SetMarginContract {
        contract: String,
    },
    RedeemFromVaultContract {
        denom: String,
        amount: Uint128,
        user: String,
    },
    BorrowFromVaultContract {
        denom: String,
        amount: Uint128,
        user: String,
    },
    Fund {},
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(String)]
    GetLendingContract,

    #[returns(String)]
    GetMarginContract,
}
