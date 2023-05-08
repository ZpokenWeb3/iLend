#[cfg(test)]
mod tests {
    use cosmwasm_std::{coin, coins, Addr};
    use cw_multi_test::{App, ContractWrapper, Executor};
    use std::vec;

    use cosmwasm_std::Uint128;
    use master_contract::msg::{ExecuteMsg, GetBalanceResponse, InstantiateMsg, QueryMsg};
    use master_contract::{execute, instantiate, query};

    #[test]
    fn test_successful_deposits_of_diff_token() {
        const DECIMAL_FRACTIONAL: Uint128 = Uint128::new(1_000_000_000_000_000_000u128); // 1*10**18

        const INIT_BALANCE_FIRST_TOKEN: u128 = 1000 * DECIMAL_FRACTIONAL.u128();
        const INIT_BALANCE_SECOND_TOKEN: u128 = 1000 * DECIMAL_FRACTIONAL.u128();

        const DEPOSIT_OF_FIRST_TOKEN: u128 = 200 * DECIMAL_FRACTIONAL.u128();
        const DEPOSIT_OF_SECOND_TOKEN: u128 = 300 * DECIMAL_FRACTIONAL.u128();

        const CONTRACT_RESERVES_FIRST_TOKEN: u128 = 1000 * DECIMAL_FRACTIONAL.u128();
        const CONTRACT_RESERVES_SECOND_TOKEN: u128 = 1000 * DECIMAL_FRACTIONAL.u128();

        const MIN_INTEREST_RATE: u128 = 5u128 * DECIMAL_FRACTIONAL.u128();
        const SAFE_BORROW_MAX_RATE: u128 = 30u128 * DECIMAL_FRACTIONAL.u128();
        const RATE_GROWTH_FACTOR: u128 = 70u128 * DECIMAL_FRACTIONAL.u128();

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
                .unwrap();

            router
                .bank
                .init_balance(
                    storage,
                    &Addr::unchecked("owner"),
                    vec![
                        coin(CONTRACT_RESERVES_FIRST_TOKEN, "eth"),
                        coin(CONTRACT_RESERVES_SECOND_TOKEN, "atom"),
                    ],
                )
                .unwrap();
        });

        let code = ContractWrapper::new(execute, instantiate, query);
        let code_id = app.store_code(Box::new(code));

        let addr = app
            .instantiate_contract(
                code_id,
                Addr::unchecked("owner"),
                &InstantiateMsg {
                    admin: "owner".to_string(),
                    supported_tokens: vec![
                        (
                            "eth".to_string(),
                            "ethereum".to_string(),
                            "ETH".to_string(),
                            18,
                        ),
                        (
                            "atom".to_string(),
                            "atom".to_string(),
                            "ATOM".to_string(),
                            18,
                        ),
                    ],
                    tokens_interest_rate_model_params: vec![
                        (
                            "eth".to_string(),
                            MIN_INTEREST_RATE,
                            SAFE_BORROW_MAX_RATE,
                            RATE_GROWTH_FACTOR,
                        ),
                        (
                            "atom".to_string(),
                            MIN_INTEREST_RATE,
                            SAFE_BORROW_MAX_RATE,
                            RATE_GROWTH_FACTOR,
                        ),
                    ],
                },
                &[coin(CONTRACT_RESERVES_SECOND_TOKEN, "atom")],
                "Contract",
                Some("owner".to_string()), // contract that can execute migrations
            )
            .unwrap();

        // funding contract with second reserve
        app.execute_contract(
            Addr::unchecked("owner"),
            addr.clone(),
            &ExecuteMsg::Fund {},
            &coins(CONTRACT_RESERVES_FIRST_TOKEN, "eth"),
        )
        .unwrap();

        app.execute_contract(
            Addr::unchecked("user"),
            addr.clone(),
            &ExecuteMsg::Deposit {},
            &coins(DEPOSIT_OF_FIRST_TOKEN, "eth"),
        )
        .unwrap();

        let user_deposited_balance: GetBalanceResponse = app
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
            user_deposited_balance.balance.u128(),
            DEPOSIT_OF_FIRST_TOKEN
        );

        assert_eq!(
            app.wrap()
                .query_balance("user", "eth")
                .unwrap()
                .amount
                .u128(),
            INIT_BALANCE_FIRST_TOKEN - DEPOSIT_OF_FIRST_TOKEN
        );

        assert_eq!(
            app.wrap()
                .query_balance(&addr, "eth")
                .unwrap()
                .amount
                .u128(),
            CONTRACT_RESERVES_FIRST_TOKEN + DEPOSIT_OF_FIRST_TOKEN
        );

        app.execute_contract(
            Addr::unchecked("user"),
            addr.clone(),
            &ExecuteMsg::Deposit {},
            &coins(DEPOSIT_OF_SECOND_TOKEN, "atom"),
        )
        .unwrap();

        let user_deposited_balance: GetBalanceResponse = app
            .wrap()
            .query_wasm_smart(
                addr.clone(),
                &QueryMsg::GetDeposit {
                    address: "user".to_string(),
                    denom: "atom".to_string(),
                },
            )
            .unwrap();

        assert_eq!(
            user_deposited_balance.balance.u128(),
            DEPOSIT_OF_SECOND_TOKEN
        );

        assert_eq!(
            app.wrap()
                .query_balance("user", "atom")
                .unwrap()
                .amount
                .u128(),
            INIT_BALANCE_SECOND_TOKEN - DEPOSIT_OF_SECOND_TOKEN
        );

        assert_eq!(
            app.wrap()
                .query_balance(&addr, "atom")
                .unwrap()
                .amount
                .u128(),
            CONTRACT_RESERVES_SECOND_TOKEN + DEPOSIT_OF_SECOND_TOKEN
        );
    }
}
