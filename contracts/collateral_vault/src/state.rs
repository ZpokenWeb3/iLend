use {
    cosmwasm_std::Uint128,
    cw_storage_plus::{Item, Map},
};

// accounts that are allowed to initiate deposits and withdrawals
pub const LENDING: Item<String> = Item::new("lending");
pub const MARGIN_POSITIONS: Item<String> = Item::new("margin_positions");

// contract itself, or other contract that are allowed to alter sensitive information
pub const ADMIN: Item<String> = Item::new("admin");



