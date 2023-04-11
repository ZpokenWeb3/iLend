use std::vec;

use cosmwasm_std::{
    to_binary, BankMsg, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult, Uint128,
};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::InstantiateMsg;
use crate::msg::{ExecuteMsg, QueryMsg};
use crate::state::{USER_PROFILES, VAULT_CONTRACT};

#[cfg(test)]
mod tests {
    use super::*;

    use crate::{execute, instantiate, query};
    use cosmwasm_schema::schemars::schema::SingleOrVec::Vec;
    use cosmwasm_std::{coin, coins, Addr};
    use cw_multi_test::{App, ContractWrapper, Executor};

    #[test]
    fn test_successful_deposits_of_one_token() {
        const INIT_BALANCE: u128 = 1000;
        const FIRST_DEPOSIT_AMOUNT: u128 = 200;
        const SECOND_DEPOSIT_AMOUNT: u128 = 300;

        let mut app = App::new(|router, _, storage| {
            router
                .bank
                .init_balance(
                    storage,
                    &Addr::unchecked("user"),
                    coins(INIT_BALANCE, "eth"),
                )
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
            &coins(FIRST_DEPOSIT_AMOUNT, "eth"),
        )
        .unwrap();

        let user_deposited_balance: Uint128 = app
            .wrap()
            .query_wasm_smart(
                addr.clone(),
                &QueryMsg::GetDeposit {
                    address: "user".to_string(),
                    denom: "eth".to_string(),
                },
            )
            .unwrap();

        assert_eq!(user_deposited_balance.u128(), FIRST_DEPOSIT_AMOUNT);

        assert_eq!(
            app.wrap()
                .query_balance("user", "eth")
                .unwrap()
                .amount
                .u128(),
            INIT_BALANCE - FIRST_DEPOSIT_AMOUNT
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
            FIRST_DEPOSIT_AMOUNT
        );

        app.execute_contract(
            Addr::unchecked("user"),
            addr.clone(),
            &ExecuteMsg::Deposit {},
            &coins(SECOND_DEPOSIT_AMOUNT, "eth"),
        )
        .unwrap();

        let user_deposited_balance: Uint128 = app
            .wrap()
            .query_wasm_smart(
                addr.clone(),
                &QueryMsg::GetDeposit {
                    address: "user".to_string(),
                    denom: "eth".to_string(),
                },
            )
            .unwrap();

        assert_eq!(
            user_deposited_balance.u128(),
            FIRST_DEPOSIT_AMOUNT + SECOND_DEPOSIT_AMOUNT
        );

        assert_eq!(
            app.wrap()
                .query_balance("user", "eth")
                .unwrap()
                .amount
                .u128(),
            INIT_BALANCE - FIRST_DEPOSIT_AMOUNT - SECOND_DEPOSIT_AMOUNT
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
            FIRST_DEPOSIT_AMOUNT + SECOND_DEPOSIT_AMOUNT
        );
    }

    #[test]
    fn test_fail_deposit_insufficient_balance() {
        const INIT_BALANCE: u128 = 100;
        const DEPOSIT_AMOUNT: u128 = 200;

        let mut app = App::new(|router, _, storage| {
            router
                .bank
                .init_balance(
                    storage,
                    &Addr::unchecked("user"),
                    coins(INIT_BALANCE, "eth"),
                )
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

        assert!(app
            .execute_contract(
                Addr::unchecked("user"),
                addr.clone(),
                &ExecuteMsg::Deposit {},
                &coins(DEPOSIT_AMOUNT, "eth"),
            )
            .is_err());

        let user_deposited_balance: Uint128 = app
            .wrap()
            .query_wasm_smart(
                addr.clone(),
                &QueryMsg::GetDeposit {
                    address: "user".to_string(),
                    denom: "eth".to_string(),
                },
            )
            .unwrap();

        assert_eq!(user_deposited_balance.u128(), 0);
    }

    #[test]
    fn test_successful_deposits_of_diff_token() {
        const INIT_BALANCE_FIRST_TOKEN: u128 = 1000;
        const INIT_BALANCE_SECOND_TOKEN: u128 = 1000;

        const DEPOSIT_OF_FIRST_TOKEN: u128 = 200;
        const DEPOSIT_OF_SECOND_TOKEN: u128 = 300;

        let mut app = App::new(|router, _, storage| {
            router
                .bank
                .init_balance(
                    storage,
                    &Addr::unchecked("user"),
                    vec![
                        coin(INIT_BALANCE_FIRST_TOKEN, "eth"),
                        coin(INIT_BALANCE_SECOND_TOKEN, "atom"),
                    ],
                )
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
            &coins(DEPOSIT_OF_FIRST_TOKEN, "eth"),
        )
        .unwrap();

        let user_deposited_balance: Uint128 = app
            .wrap()
            .query_wasm_smart(
                addr.clone(),
                &QueryMsg::GetDeposit {
                    address: "user".to_string(),
                    denom: "eth".to_string(),
                },
            )
            .unwrap();

        assert_eq!(user_deposited_balance.u128(), DEPOSIT_OF_FIRST_TOKEN);

        assert_eq!(
            app.wrap()
                .query_balance("user", "eth")
                .unwrap()
                .amount
                .u128(),
            INIT_BALANCE_FIRST_TOKEN - DEPOSIT_OF_FIRST_TOKEN
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
            DEPOSIT_OF_FIRST_TOKEN
        );

        app.execute_contract(
            Addr::unchecked("user"),
            addr.clone(),
            &ExecuteMsg::Deposit {},
            &coins(DEPOSIT_OF_SECOND_TOKEN, "atom"),
        )
        .unwrap();

        let user_deposited_balance: Uint128 = app
            .wrap()
            .query_wasm_smart(
                addr.clone(),
                &QueryMsg::GetDeposit {
                    address: "user".to_string(),
                    denom: "atom".to_string(),
                },
            )
            .unwrap();

        assert_eq!(user_deposited_balance.u128(), DEPOSIT_OF_SECOND_TOKEN);

        assert_eq!(
            app.wrap()
                .query_balance("user", "atom")
                .unwrap()
                .amount
                .u128(),
            INIT_BALANCE_SECOND_TOKEN - DEPOSIT_OF_SECOND_TOKEN
        );

        // as our contract don't store it, should be ZERO
        assert_eq!(
            app.wrap()
                .query_balance(&addr, "atom")
                .unwrap()
                .amount
                .u128(),
            0
        );

        assert_eq!(
            app.wrap()
                .query_balance("vault_contract", "atom")
                .unwrap()
                .amount
                .u128(),
            DEPOSIT_OF_SECOND_TOKEN
        );
    }
}
