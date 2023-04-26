#[cfg(test)]
mod tests {
    use cosmwasm_std::{coin, coins, Addr};
    use cw_multi_test::{App, ContractWrapper, Executor};
    use std::vec;

    use cosmwasm_std::Uint128;
    use master_contract::msg::{
        ExecuteMsg, GetBalanceResponse, GetTotalDepositedUsdResponse, InstantiateMsg, QueryMsg,
    };
    use master_contract::{execute, instantiate, query};

    #[test]
    fn test_successful_deposits_of_one_token() {
        const TOKEN_DECIMALS: u32 = 6;

        const INIT_USER_BALANCE: u128 = 1000u128 * 10u128.pow(TOKEN_DECIMALS);
        const CONTRACT_RESERVES: u128 = 1000000u128 * 10u128.pow(TOKEN_DECIMALS);
        const FIRST_DEPOSIT_AMOUNT: u128 = 200u128 * 10u128.pow(TOKEN_DECIMALS);
        const SECOND_DEPOSIT_AMOUNT: u128 = 300u128 * 10u128.pow(TOKEN_DECIMALS);

        const INTEREST_RATE_DECIMALS: u32 = 18;

        const MIN_INTEREST_RATE: u128 = 5u128 * 10u128.pow(INTEREST_RATE_DECIMALS);
        const SAFE_BORROW_MAX_RATE: u128 = 30u128 * 10u128.pow(INTEREST_RATE_DECIMALS);
        const RATE_GROWTH_FACTOR: u128 = 70u128 * 10u128.pow(INTEREST_RATE_DECIMALS);

        let mut app = App::new(|router, _, storage| {
            router
                .bank
                .init_balance(
                    storage,
                    &Addr::unchecked("user"),
                    coins(INIT_USER_BALANCE, "eth"),
                )
                .unwrap();

            router
                .bank
                .init_balance(
                    storage,
                    &Addr::unchecked("owner"),
                    coins(CONTRACT_RESERVES, "eth"),
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
                    admin: "owner".to_string(),
                    supported_tokens: vec![],
                    tokens_interest_rate_model_params: vec![],
                },
                &[coin(CONTRACT_RESERVES, "eth")],
                "Contract",
                Some("owner".to_string()), // contract that can execute migrations
            )
            .unwrap();

        app.execute_contract(
            Addr::unchecked("user"),
            addr.clone(),
            &ExecuteMsg::AddMarkets {
                denom: "eth".to_string(),
                name: "ethereum".to_string(),
                symbol: "ETH".to_string(),
                decimals: TOKEN_DECIMALS as u128,
                min_interest_rate: MIN_INTEREST_RATE,
                safe_borrow_max_rate: SAFE_BORROW_MAX_RATE,
                rate_growth_factor: RATE_GROWTH_FACTOR,
            },
            &[],
        )
        .unwrap();

        app.execute_contract(
            Addr::unchecked("user"),
            addr.clone(),
            &ExecuteMsg::Deposit {},
            &coins(FIRST_DEPOSIT_AMOUNT, "eth"),
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

        assert_eq!(user_deposited_balance.balance.u128(), FIRST_DEPOSIT_AMOUNT);

        assert_eq!(
            app.wrap()
                .query_balance("user", "eth")
                .unwrap()
                .amount
                .u128(),
            INIT_USER_BALANCE - FIRST_DEPOSIT_AMOUNT
        );

        assert_eq!(
            app.wrap()
                .query_balance(&addr, "eth")
                .unwrap()
                .amount
                .u128(),
            CONTRACT_RESERVES + FIRST_DEPOSIT_AMOUNT
        );

        app.execute_contract(
            Addr::unchecked("user"),
            addr.clone(),
            &ExecuteMsg::Deposit {},
            &coins(SECOND_DEPOSIT_AMOUNT, "eth"),
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

        let available_to_redeem_another_token: Uint128 = app
            .wrap()
            .query_wasm_smart(
                addr.clone(),
                &QueryMsg::GetAvailableToRedeem {
                    address: "user".to_string(),
                    denom: "atom".to_string(),
                },
            )
            .unwrap();

        assert_eq!(available_to_redeem_another_token.u128(), 0);

        assert_eq!(
            user_deposited_balance.balance.u128(),
            FIRST_DEPOSIT_AMOUNT + SECOND_DEPOSIT_AMOUNT
        );

        assert_eq!(
            app.wrap()
                .query_balance("user", "eth")
                .unwrap()
                .amount
                .u128(),
            INIT_USER_BALANCE - FIRST_DEPOSIT_AMOUNT - SECOND_DEPOSIT_AMOUNT
        );

        assert_eq!(
            app.wrap()
                .query_balance(&addr, "eth")
                .unwrap()
                .amount
                .u128(),
            CONTRACT_RESERVES + FIRST_DEPOSIT_AMOUNT + SECOND_DEPOSIT_AMOUNT
        );
    }
}
