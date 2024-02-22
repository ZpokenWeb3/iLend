use cosmwasm_std::StdError;
use cw_utils::PaymentError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Allowed for Admin Only")]
    ForAdminOnly {},

    #[error("Allowed for Price Updater Contract Only")]
    ForPriceUpdaterContractOnly {},

    #[error("Custom Error val: {val:?}")]
    CustomError { val: String },

    #[error("InvalidFunds: {msg}")]
    InvalidFunds { msg: String },

    #[error("{0}")]
    Payment(#[from] PaymentError),

    #[error("CoinNotFound")]
    CoinNotFound {},

    #[error("Token Not Supported")]
    TokenNotSupported {},

    #[error("Invalid Asset: {asset}")]
    InvalidAsset { asset: String },

    #[error("Not Enough Balance To Do Redeem")]
    NotEnoughBalanceToDoRedeem {},

    #[error("Amount To Be Borrowed Is Not Available")]
    AmountToBeBorrowedIsNotAvailable {},

    #[error("Not Enough Liquidity To Borrow")]
    NotEnoughLiquidityToBorrow {},
}
