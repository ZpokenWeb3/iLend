use cosmwasm_std::{CosmosMsg, WasmMsg};

use {
    crate::contract::query::get_balance,
    crate::{
        error::ContractError,
        msg::InstantiateMsg,
        msg::{ExecuteMsg, QueryMsg},
        state::{ADMIN, SUPPORTED_TOKENS, USER_PROFILES},
    },
    cosmwasm_std::{
        coins, to_binary, BankMsg, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult,
        Uint128,
    },
    cw2::set_contract_version,
    cw20::Cw20ExecuteMsg,
};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:master_contract";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    ADMIN.save(deps.storage, &msg.admin)?;

    for tokens in msg.supported_tokens {
        SUPPORTED_TOKENS.save(deps.storage, tokens.0, &tokens.1)?;
    }

    Ok(Response::new().add_attribute("method", "instantiate"))
}

pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Deposit {} => {
            // underlying balances that are allowing user to send funds along with calling Deposit
            // is operated through cw20 contracts

            if info.funds.is_empty() {
                return Err(ContractError::CustomError {
                    val: "No funds deposited!".to_string(),
                });
            }

            assert_eq!(
                info.funds.len(),
                1,
                "You have to deposit one asset per time"
            );

            let allowed_coin = info.funds.first().unwrap();

            assert!(allowed_coin.amount.u128() > 0);

            assert!(
                !SUPPORTED_TOKENS
                    .load(deps.storage, allowed_coin.denom.clone())
                    .unwrap()
                    .is_empty(),
                "There is no such supported token yet"
            );

            let current_balance = get_balance(
                deps.as_ref(),
                info.sender.to_string(),
                allowed_coin.denom.clone(),
            )
            .unwrap();
            let new_balance = current_balance.balance.u128() + allowed_coin.amount.u128();
            USER_PROFILES.save(
                deps.storage,
                (info.sender.to_string(), allowed_coin.denom.clone()),
                &Uint128::from(new_balance),
            )?;

            let mint_binary_msg = to_binary(&Cw20ExecuteMsg::Mint {
                recipient: info.sender.to_string(),
                amount: allowed_coin.amount.clone(),
            })?;

            Ok(
                Response::default().add_message(CosmosMsg::Wasm(WasmMsg::Execute {
                    contract_addr: SUPPORTED_TOKENS
                        .load(deps.storage, allowed_coin.denom.clone())
                        .unwrap(),
                    msg: mint_binary_msg,
                    funds: vec![],
                })),
            )
        }
        ExecuteMsg::Redeem { denom, amount } => {
            // TODO have to have exact amount of itokens transfered in info.funds along the call

            if info.funds.is_empty() {
                return Err(ContractError::CustomError {
                    val: "No LP tokens deposited!".to_string(),
                });
            }

            assert!(amount.u128() > 0, "Amount should be a positive number");

            assert!(
                !SUPPORTED_TOKENS
                    .load(deps.storage, denom.clone())
                    .unwrap()
                    .is_empty(),
                "There is no such supported token yet"
            );

            let current_balance =
                query::get_balance(deps.as_ref(), info.sender.to_string(), denom.clone())?;

            // the ratio of token an mmtoken
            let xrate = 1u128;

            let amount = amount.u128() * xrate;

            assert!(
                current_balance.balance.u128() >= amount,
                "The account doesn't have enough digital tokens to do withdraw"
            );

            let remaining = current_balance.balance.u128() - amount;

            USER_PROFILES.save(
                deps.storage,
                (info.sender.to_string(), denom.clone()),
                &Uint128::from(remaining),
            )?;

            let burn_binary_msg = to_binary(&Cw20ExecuteMsg::Burn {
                amount: Uint128::from(current_balance.balance.u128() - amount),
            })?;

            Ok(Response::new()
                .add_message(BankMsg::Send {
                    to_address: info.sender.to_string(),
                    amount: coins(amount, denom.clone()),
                })
                .add_message(
                    // burning received amount of mmTokens
                    CosmosMsg::Wasm(WasmMsg::Execute {
                        contract_addr: SUPPORTED_TOKENS.load(deps.storage, denom.clone()).unwrap(),
                        msg: burn_binary_msg,
                        funds: vec![],
                    }),
                ))
        }
        ExecuteMsg::Fund {} => {
            if info.funds.is_empty() {
                return Err(ContractError::CustomError {
                    val: "No funds deposited!".to_string(),
                });
            }

            assert_eq!(
                info.sender.to_string(),
                ADMIN.load(deps.storage).unwrap(),
                "This functionality is allowed for admin only"
            );

            Ok(Response::default())
        }
        ExecuteMsg::AddMarkets { token, itoken } => {
            SUPPORTED_TOKENS.save(deps.storage, token, &itoken)?;

            Ok(Response::default())
        }
    }
}

pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetDeposit { address, denom } => to_binary(&get_balance(deps, address, denom)?),
    }
}

pub mod query {

    use super::*;
    use crate::msg::GetBalanceResponse;

    pub fn get_balance(
        deps: Deps,
        address: String,
        denom: String,
    ) -> StdResult<GetBalanceResponse> {
        let balance = USER_PROFILES
            .load(deps.storage, (address, denom))
            .unwrap_or_else(|_| Uint128::zero());

        Ok(GetBalanceResponse { balance })
    }
}
