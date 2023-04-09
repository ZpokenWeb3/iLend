use std::{vec};

use cosmwasm_std::{Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult, Uint128, BankMsg, Addr, Coin};
use cw2::set_contract_version;

use crate::msg::{ExecuteMsg, QueryMsg};
use crate::error::ContractError;
use crate::msg::InstantiateMsg;
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


pub fn execute(deps: DepsMut, _env: Env, info: MessageInfo, msg: ExecuteMsg) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Deposit {} => {
            // TODO add some checks for the corresponding underlying balances for user


            for coin in &info.funds {
                let current_balance = get_balance(deps.as_ref(), info.sender.clone(), coin.denom.clone())?;
                let new_balance = current_balance + coin.amount;
                USER_PROFILES.save(deps.storage, (info.sender.to_string(), coin.denom.clone()), &new_balance)?;
            }

            // sending funds to the vault contract
            let msg = vec![BankMsg::Send {
                to_address: VAULT_CONTRACT.load(deps.storage).unwrap(),
                amount: info.funds,
            }];

            Ok(Response::new()
                .add_messages(msg))
        }
    }
}

fn get_balance(deps: Deps, address: Addr, token: String) -> StdResult<Uint128> {
    let balance = USER_PROFILES.load(deps.storage, (address.to_string(), token)).unwrap_or_else(|_| Uint128::zero());

    Ok(balance)
}

pub fn query(
    _deps: Deps,
    _env: Env,
    _msg: QueryMsg,
) -> StdResult<Binary> {
    unimplemented!()
}


#[cfg(test)]
mod tests {
    use cosmwasm_std::Addr;
    use cw_multi_test::{App, ContractWrapper, Executor};
    use crate::msg::ExecuteMsg::Deposit;


    use super::*;

    #[test]
    fn test_deposit() {
        const INIT_BALANCE: u128 = 1000;
        const DEPOSIT_AMOUNT: u128 = 200;

        let mut app = App::new(|router, _, storage| {
            router
                .bank
                .init_balance(storage, &Addr::unchecked("user"), coins(INIT_BALANCE, "eth"))
                .unwrap()
        });

        let code = ContractWrapper::new(execute, instantiate, query);
        let code_id = app.store_code(Box::new(code));

        let addr = app
            .instantiate_contract(
                code_id,
                Addr::unchecked("owner"),
                &InstantiateMsg {
                    vault: "vault_contract".to_owned(),
                    denom: "eth".to_owned(),
                },
                &[],
                "Contract",
                None,
            )
            .unwrap();

        app.execute_contract(
            Addr::unchecked("user"),
            addr.clone(),
            &ExecuteMsg::Deposit {},
            &coins(DEPOSIT_AMOUNT, "eth"),
        )
            .unwrap();

        assert_eq!(
            app.wrap()
                .query_balance("user", "eth")
                .unwrap()
                .amount
                .u128(),
            INIT_BALANCE - DEPOSIT_AMOUNT
        );

        // as our contract don't store it, should be ZERO
        assert_eq!(
            app.wrap()
                .query_balance(&addr, "eth")
                .unwrap()
                .amount
                .u128(),
            0
        );

        assert_eq!(
            app.wrap()
                .query_balance("vault_contract", "eth")
                .unwrap()
                .amount
                .u128(),
            DEPOSIT_AMOUNT
        );
    }
}