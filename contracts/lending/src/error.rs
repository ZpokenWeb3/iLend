use cosmwasm_std::StdError;
use cw_utils::PaymentError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Custom Error val: {val:?}")]
    CustomError { val: String },

    #[error("InvalidFunds: {msg}")]
    InvalidFunds { msg: String },

    #[error("{0}")]
    Payment(#[from] PaymentError),

    #[error("CoinNotFound")]
    CoinNotFound {},

    #[error("Token not supported")]
    TokenNotSupported {},

    #[error("Invalid Asset: {asset}")]
    InvalidAsset { asset: String },

    #[error("Not Enough Balance To Do Redeem")]
    NotEnoughBalanceToDoRedeem {}
}
