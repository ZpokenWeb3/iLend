#[cfg(test)]
mod tests {
    use cosmwasm_std::{coin, coins, Addr, BlockInfo, Timestamp, Uint128};

    use cw_multi_test::{App, ContractWrapper, Executor};
    use std::vec;

    use lending::msg::{ExecuteMsg, GetBalanceResponse, InstantiateMsg, QueryMsg, TotalBorrowData};
    use lending::{execute, instantiate, query};
    use pyth_sdk_cw::PriceIdentifier;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn test_successful_deposits_of_one_token() {
        const TOKENS_DECIMALS: u32 = 18;

        const INIT_USER_BALANCE: u128 = 1000 * 10u128.pow(TOKENS_DECIMALS);
        const INIT_LIQUIDATOR_BALANCE_ETH: u128 = 1_000_000 * 10u128.pow(TOKENS_DECIMALS); // 1M ETH
        const INIT_LIQUIDATOR_BALANCE_ATOM: u128 = 1_000_000 * 10u128.pow(TOKENS_DECIMALS); // 1M ATOM

        const CONTRACT_RESERVES: u128 = 1000000 * 10u128.pow(TOKENS_DECIMALS);
        const FIRST_DEPOSIT_AMOUNT: u128 = 200 * 10u128.pow(TOKENS_DECIMALS);
        const SECOND_DEPOSIT_AMOUNT: u128 = 300 * 10u128.pow(TOKENS_DECIMALS);

        const PERCENT_DECIMALS: u32 = 5;
        const LTV_ETH: u128 = 85 * 10u128.pow(PERCENT_DECIMALS); // 85%
        const LIQUIDATION_THRESHOLD_ETH: u128 = 90 * 10u128.pow(PERCENT_DECIMALS); // 90%
        const LTV_ATOM: u128 = 75 * 10u128.pow(PERCENT_DECIMALS); // 75%
        const LIQUIDATION_THRESHOLD_ATOM: u128 = 80 * 10u128.pow(PERCENT_DECIMALS); // 80%

        const INTEREST_RATE_DECIMALS: u32 = 18;
        const MIN_INTEREST_RATE: u128 = 5 * 10u128.pow(INTEREST_RATE_DECIMALS);
        const SAFE_BORROW_MAX_RATE: u128 = 30 * 10u128.pow(INTEREST_RATE_DECIMALS);
        const RATE_GROWTH_FACTOR: u128 = 70 * 10u128.pow(INTEREST_RATE_DECIMALS);

        const OPTIMAL_UTILISATION_RATIO: u128 = 80 * 10u128.pow(PERCENT_DECIMALS);

        const PRICE_DECIMALS: u32 = 8;
        const PRICE_ETH: u128 = 2000 * 10u128.pow(PRICE_DECIMALS);
        const PRICE_ATOM: u128 = 10 * 10u128.pow(PRICE_DECIMALS);

        let mut app = App::new(|router, _, storage| {
            router
                .bank
                .init_balance(
                    storage,
                    &Addr::unchecked("user"),
                    vec![
                        coin(INIT_USER_BALANCE, "eth"),
                        coin(INIT_USER_BALANCE, "atom"),
                    ],
                )
                .unwrap();

            router
                .bank
                .init_balance(
                    storage,
                    &Addr::unchecked("owner"),
                    vec![
                        coin(CONTRACT_RESERVES * 100, "eth"),
                        coin(CONTRACT_RESERVES * 100, "atom"),
                    ],
                )
                .unwrap();

            router
                .bank
                .init_balance(
                    storage,
                    &Addr::unchecked("liquidator"),
                    vec![
                        coin(INIT_LIQUIDATOR_BALANCE_ETH, "eth"),
                        coin(INIT_LIQUIDATOR_BALANCE_ATOM, "atom"),
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
                    is_testing: true,
                    admin: "owner".to_string(),
                    price_ids: vec![
                        (
                            "inj".to_string(),
                            PriceIdentifier::from_hex(
                                "2d9315a88f3019f8efa88dfe9c0f0843712da0bac814461e27733f6b83eb51b3",
                            )
                            .unwrap(),
                        ),
                        (
                            "peggy0x44C21afAaF20c270EBbF5914Cfc3b5022173FEB7".to_string(),
                            PriceIdentifier::from_hex(
                                "2d9315a88f3019f8efa88dfe9c0f0843712da0bac814461e27733f6b83eb51b3",
                            )
                            .unwrap(),
                        ),
                    ],
                    pyth_contract_addr: "inj1z60tg0tekdzcasenhuuwq3htjcd5slmgf7gpez".to_string(),
                    supported_tokens: vec![(
                        "atom".to_string(),
                        "atom".to_string(),
                        "ATOM".to_string(),
                        6,
                    )],
                    reserve_configuration: vec![(
                        "atom".to_string(),
                        LTV_ATOM,
                        LIQUIDATION_THRESHOLD_ATOM,
                    )],
                    tokens_interest_rate_model_params: vec![(
                        "atom".to_string(),
                        5000000000000000000,
                        20000000000000000000,
                        100000000000000000000,
                        OPTIMAL_UTILISATION_RATIO,
                    )],
                    price_updater_addr: "".to_string(),
                },
                &[],
                "Contract",
                Some("owner".to_string()), // contract that can execute migrations
            )
            .unwrap();

        app.execute_contract(
            Addr::unchecked("owner"),
            addr.clone(),
            &ExecuteMsg::AddMarkets {
                denom: "eth".to_string(),
                name: "ethereum".to_string(),
                symbol: "ETH".to_string(),
                decimals: TOKENS_DECIMALS as u128,
                loan_to_value_ratio: LTV_ETH,
                liquidation_threshold: LIQUIDATION_THRESHOLD_ETH,
                min_interest_rate: MIN_INTEREST_RATE,
                safe_borrow_max_rate: SAFE_BORROW_MAX_RATE,
                rate_growth_factor: RATE_GROWTH_FACTOR,
                optimal_utilisation_ratio: OPTIMAL_UTILISATION_RATIO,
            },
            &[],
        )
        .unwrap();

        app.send_tokens(
            Addr::unchecked("owner"),
            addr.clone(),
            &coins(CONTRACT_RESERVES / 10, "eth"),
        )
            .unwrap();

        app.send_tokens(
            Addr::unchecked("owner"),
            addr.clone(),
            &coins(CONTRACT_RESERVES / 10, "atom"),
        )
            .unwrap();

        app.execute_contract(
            Addr::unchecked("owner"),
            addr.clone(),
            &ExecuteMsg::UpdatePrice {
                denom: Some("eth".to_string()),
                price: Some(PRICE_ETH),
            },
            &[],
        )
        .unwrap();

        app.execute_contract(
            Addr::unchecked("owner"),
            addr.clone(),
            &ExecuteMsg::UpdatePrice {
                denom: Some("atom".to_string()),
                price: Some(PRICE_ATOM),
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

        app.execute_contract(
            Addr::unchecked("owner"),
            addr.clone(),
            &ExecuteMsg::Deposit {},
            &coins(FIRST_DEPOSIT_AMOUNT * 15 / 10, "eth"),
        )
        .unwrap();

        app.execute_contract(
            Addr::unchecked("user"),
            addr.clone(),
            &ExecuteMsg::ToggleCollateralSetting {
                denom: "eth".to_string(),
            },
            &[],
        )
        .unwrap();

        app.execute_contract(
            Addr::unchecked("owner"),
            addr.clone(),
            &ExecuteMsg::ToggleCollateralSetting {
                denom: "eth".to_string(),
            },
            &[],
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

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        app.set_block(BlockInfo {
            height: 0,
            time: Timestamp::from_seconds(now),
            chain_id: "custom_chain_id".to_string(),
        });

        app.execute_contract(
            Addr::unchecked("user"),
            addr.clone(),
            &ExecuteMsg::Deposit {},
            &coins(SECOND_DEPOSIT_AMOUNT, "eth"),
        )
        .unwrap();

        app.execute_contract(
            Addr::unchecked("owner"),
            addr.clone(),
            &ExecuteMsg::Borrow {
                denom: "eth".to_string(),
                amount: Uint128::from(SECOND_DEPOSIT_AMOUNT / 2),
            },
            &[],
        )
        .unwrap();

        app.set_block(BlockInfo {
            height: 0,
            time: Timestamp::from_seconds(now + 31536000),
            chain_id: "custom_chain_id".to_string(),
        });

        let total_borrow_data: TotalBorrowData = app
            .wrap()
            .query_wasm_smart(
                addr.clone(),
                &QueryMsg::GetTotalBorrowData {
                    denom: "eth".to_string(),
                },
            )
            .unwrap();

        let reserves_by_token: Uint128 = app
            .wrap()
            .query_wasm_smart(
                addr.clone(),
                &QueryMsg::GetTotalReservesByToken {
                    denom: "eth".to_string(),
                },
            )
            .unwrap();

        let liquidity_rate: Uint128 = app
            .wrap()
            .query_wasm_smart(
                addr.clone(),
                &QueryMsg::GetLiquidityRate {
                    denom: "eth".to_string(),
                },
            )
            .unwrap();

        let borrow_amount_with_interest: Uint128 = app
            .wrap()
            .query_wasm_smart(
                addr.clone(),
                &QueryMsg::GetUserBorrowAmountWithInterest {
                    address: "owner".to_string(),
                    denom: "eth".to_string(),
                },
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

        let price: Uint128 = app
            .wrap()
            .query_wasm_smart(
                addr.clone(),
                &QueryMsg::GetMmTokenPrice {
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

        assert!(
            user_deposited_balance.balance.u128() > FIRST_DEPOSIT_AMOUNT + SECOND_DEPOSIT_AMOUNT
        );

        assert_eq!(
            app.wrap()
                .query_balance("user", "eth")
                .unwrap()
                .amount
                .u128(),
            INIT_USER_BALANCE - FIRST_DEPOSIT_AMOUNT - SECOND_DEPOSIT_AMOUNT
        );
    }
}
