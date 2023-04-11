use std::vec;

use cosmwasm_std::{to_binary, BankMsg, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult, Uint128, coins};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::InstantiateMsg;
use crate::msg::{ExecuteMsg, QueryMsg};
use crate::state::{ADMIN, SUPPORTED_TOKENS, USER_PROFILES};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:master_contract";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    // make sure we are sending some reserves in supported tokens to be able to cover first trades for early users
    assert!(!info.funds.is_empty());

    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    ADMIN.save(deps.storage, &msg.admin)?;

    for tokens in msg.supported_tokens {
        SUPPORTED_TOKENS.save(deps.storage, tokens.0, &tokens.1)?;
    }

    Ok(Response::default())
}

pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Deposit {} => {
            // TODO add some checks for the corresponding underlying balances for user

            for coin in &info.funds {
                let current_balance =
                    query::get_balance(deps.as_ref(), info.sender.to_string(), coin.denom.clone())?;
                let new_balance = current_balance + coin.amount;
                USER_PROFILES.save(
                    deps.storage,
                    (info.sender.to_string(), coin.denom.clone()),
                    &new_balance,
                )?;
            }

            Ok(Response::default())
        }
        ExecuteMsg::Withdraw {
            denom,
            amount,
        } => {
            // TODO have to have exact amount of itokens transfered in info.funds along the call

            assert!(
                amount.u128() > 0,
                "Amount should be a positive number"
            );

            let user_balance =
                query::get_balance(deps.as_ref(), info.sender.to_string(), denom.clone())?;

            assert!(user_balance.u128() >= amount.u128(), "The account doesn't have enough digital tokens to do withdraw");

            let msg = vec![BankMsg::Send {
                to_address: info.sender.to_string(),
                amount: coins(amount.u128(), denom),
            }];

            // TODO burn respective amount of itokens here
            Ok(Response::new().add_messages(msg))
        }
        ExecuteMsg::Fund {} => {
            assert_eq!(info.sender.to_string(), ADMIN.load(deps.storage).unwrap(), "This functionality is allowed for admin only");

            Ok(Response::default())
        }
    }
}

pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetDeposit { address, denom } => {
            to_binary(&query::get_balance(deps, address, denom)?)
        }
    }
}

mod query {
    use super::*;


    pub fn get_balance(deps: Deps, address: String, denom: String) -> StdResult<Uint128> {
        let balance = USER_PROFILES
            .load(deps.storage, (address, denom))
            .unwrap_or_else(|_| Uint128::zero());

        Ok(balance)
    }
}
