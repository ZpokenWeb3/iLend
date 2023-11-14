use cw_storage_plus::Item;

// accounts that are allowed to initiate deposits and withdrawals
pub const LENDING_CONTRACT: Item<String> = Item::new("lending_contract");
pub const MARGIN_POSITIONS_CONTRACT: Item<String> = Item::new("margin_positions_contract");

// contract itself, or other contract that are allowed to alter sensitive information
pub const ADMIN: Item<String> = Item::new("admin");
