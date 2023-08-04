use cosmwasm_schema::cw_serde;
use cosmwasm_std::Uint128;

#[cw_serde]
pub struct TokenInfo {
    pub denom: String,
    pub name: String,
    pub symbol: String,
    pub decimals: u128,
}

#[cw_serde]
#[serde(rename = "snake_case")]
pub enum ExecuteCollateralVaultFromMarginContract {
    RedeemFromVaultContractMargin {
        denom: String,
        amount: Uint128,
        user: String,
    },
}

#[cw_serde]
#[serde(rename = "snake_case")]
pub struct OrderInfo {
    pub order_status: OrderStatus,
    pub order_type: OrderType,
    pub amount: Uint128,
    pub sell_token_denom: String,
    // pub buy_token: AccountId,
    pub leverage: u128,
    // pub sell_token_price: Uint128,
    // pub buy_token_price: Uint128,
}

#[cw_serde]
#[serde(rename = "snake_case")]
pub enum OrderType {
    Long,
    Short,
}

#[cw_serde]
#[serde(rename = "snake_case")]
pub enum OrderStatus {
    Pending,
    // Executed,
    Canceled,
    // Closed,
    // Liquidated,
}
