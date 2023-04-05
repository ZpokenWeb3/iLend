use cosmwasm_std::{Addr, Uint128};
use cw_storage_plus::Map;

pub const BALANCES: Map<&Addr, Uint128> = Map::new("balance");
