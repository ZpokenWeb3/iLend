use std::vec;

use cosmwasm_std::{
    to_binary, BankMsg, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult, Uint128,
};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::InstantiateMsg;
use crate::msg::{ExecuteMsg, QueryMsg};
use crate::state::{USER_PROFILES, VAULT_CONTRACT};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:master_contract";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    // instantiating contract version and vault contract
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    VAULT_CONTRACT.save(deps.storage, &msg.vault)?;

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

            // sending funds to the vault contract
            let msg = vec![BankMsg::Send {
                to_address: VAULT_CONTRACT.load(deps.storage).unwrap(),
                amount: info.funds,
            }];

            Ok(Response::new().add_messages(msg))
        }
        ExecuteMsg::Withdraw {
            denom: _String,
            amount: _Uint128,
        } => {
            unimplemented!()
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
