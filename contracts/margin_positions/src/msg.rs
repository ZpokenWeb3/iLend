use cosmwasm_schema::{cw_serde, QueryResponses};

use cosmwasm_std::Uint128;

use crate::utils::{OrderStatus, OrderType};
use pyth_sdk_cw::PriceIdentifier;

#[cw_serde]
pub struct InstantiateMsg {
    pub admin: String,
    pub collateral_vault_contract: String,
    // name, denom, symbol, decimals
    pub supported_tokens: Vec<(String, String, String, u128)>,
    // vector of (token denom, price_identifier) got from https://pyth.network/developers/price-feed-ids#cosmwasm-testnet
    pub price_ids: Vec<(String, PriceIdentifier)>,
    // pyth contract on a given network that fetches prices | testnet & mainnet
    pub pyth_contract_addr: String,
    pub is_testing: bool,
    pub price_updater_contract_addr: String,
    pub lending_contract: String,
}

#[cw_serde]
pub enum ExecuteMsg {
    // if args is None, updates price via Pyth oracle, otherwise set price (only for testing)
    UpdatePrice {
        denom: Option<String>,
        price: Option<u128>,
    },
    // Deposit / Redeem functionality
    Deposit {},
    Redeem {
        denom: String,
        amount: Uint128,
    },
    SetCollateralVaultContract {
        contract: String,
    },
    CreateOrder {
        order_type: OrderType,
        amount: Uint128,
        sell_token_denom: String,
        // buy_token_denom: String,
        leverage: u128,
    },
    CancelOrder {
        order_id: u128,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(Uint128)]
    GetPrice { denom: String },

    #[returns(Uint128)]
    GetDeposit { user: String, denom: String },

    #[returns(Vec < OrderResponse >)]
    GetOrdersByUser { user: String },

    #[returns(OrderResponse)]
    GetOrderById { order_id: u128 },
}

#[cw_serde]
pub struct OrderResponse {
    pub order_status: OrderStatus,
    pub order_type: OrderType,
    pub amount: Uint128,
    pub sell_token_denom: String,
    pub leverage: u128,
}
