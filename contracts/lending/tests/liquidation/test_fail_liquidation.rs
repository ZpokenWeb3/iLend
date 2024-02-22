#[cfg(test)]
mod tests {
    use crate::utils::success_borrow_setup;
    use cosmwasm_std::{coins, Addr, BlockInfo, Timestamp, Uint128};
    use cw_multi_test::Executor;
    use lending::msg::{ExecuteMsg, GetBalanceResponse, GetReserveConfigurationResponse, QueryMsg};
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    #[should_panic]
    fn test_fail_liquidation() {
        const TOKENS_DECIMALS: u32 = 18;
        const BORROW_AMOUNT_ETH: u128 = 121 * 10u128.pow(TOKENS_DECIMALS); // 121 ETH
        const LIQUIDATOR_DEPOSIT_AMOUNT_ETH: u128 = 10_000 * 10u128.pow(TOKENS_DECIMALS); // 10_000 ETH
        const YEAR_IN_SECONDS: u64 = 31536000;

        // contract reserves: 1000 ETH
        // user deposited 200 ETH and 300 ATOM
        // user borrowed 50 ETH
        let (mut app, addr) = success_borrow_setup();

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        app.set_block(BlockInfo {
            height: 542,
            time: Timestamp::from_seconds(now + 10000),
            chain_id: "custom_chain_id".to_string(),
        });

        let user_deposited_balance_eth: GetBalanceResponse = app
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
            user_deposited_balance_eth.balance.u128(),
            200000000000000000000
        ); // 200 ETH

        let user_deposited_balance_atom: GetBalanceResponse = app
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
            user_deposited_balance_atom.balance.u128(),
            300000000000000000000
        ); // 300 ATOM

        let user_collateral_usd: Uint128 = app
            .wrap()
            .query_wasm_smart(
                addr.clone(),
                &QueryMsg::GetUserCollateralUsd {
                    address: "user".to_string(),
                },
            )
            .unwrap();

        // 200 ETH * 2000 + 300 ATOM * 10 == 403_000$
        assert_eq!(user_collateral_usd.u128(), 40300000000000);

        let reserve_configuration_response: GetReserveConfigurationResponse = app
            .wrap()
            .query_wasm_smart(addr.clone(), &QueryMsg::GetReserveConfiguration {})
            .unwrap();

        assert_eq!(
            reserve_configuration_response.reserve_configuration[0].loan_to_value_ratio,
            7500000
        ); // ltv_atom = 75%

        assert_eq!(
            reserve_configuration_response.reserve_configuration[1].loan_to_value_ratio,
            8500000
        ); // ltv_eth = 85%

        let user_max_allowed_borrow_amount_usd: Uint128 = app
            .wrap()
            .query_wasm_smart(
                addr.clone(),
                &QueryMsg::GetUserMaxAllowedBorrowAmountUsd {
                    address: "user".to_string(),
                },
            )
            .unwrap();

        // 200 ETH * 0.85 * 2000 + 300 ATOM * 0.75 * 10 == 340_000 + 2_250 = 342_250$
        assert_eq!(user_max_allowed_borrow_amount_usd.u128(), 34225000000000);

        let user_borrowed_usd: Uint128 = app
            .wrap()
            .query_wasm_smart(
                addr.clone(),
                &QueryMsg::GetUserBorrowedUsd {
                    address: "user".to_string(),
                },
            )
            .unwrap();

        assert_eq!(user_borrowed_usd.u128(), 10000000000000); // 50 ETH * 2000 = 100_000$

        let available_to_borrow_eth: Uint128 = app
            .wrap()
            .query_wasm_smart(
                addr.clone(),
                &QueryMsg::GetAvailableToBorrow {
                    address: "user".to_string(),
                    denom: "eth".to_string(),
                },
            )
            .unwrap();

        // (user_max_allowed_borrow_amount_usd - user_borrowed_usd) / price =
        // (342_250$ - 100_000$) / price = 242_250$ / price
        assert_eq!(available_to_borrow_eth.u128(), 121125000000000000000); // 242_250$ / 2000 == 121.125 ETH

        app.execute_contract(
            Addr::unchecked("user"),
            addr.clone(),
            &ExecuteMsg::Borrow {
                denom: "eth".to_string(),
                amount: Uint128::from(BORROW_AMOUNT_ETH),
            },
            &[],
        )
        .unwrap();

        let available_to_borrow_eth: Uint128 = app
            .wrap()
            .query_wasm_smart(
                addr.clone(),
                &QueryMsg::GetAvailableToBorrow {
                    address: "user".to_string(),
                    denom: "eth".to_string(),
                },
            )
            .unwrap();

        assert_eq!(available_to_borrow_eth.u128(), 125000000000000000); // 0.125 ETH

        let user_liquidation_threshold: Uint128 = app
            .wrap()
            .query_wasm_smart(
                addr.clone(),
                &QueryMsg::GetUserLiquidationThreshold {
                    address: "user".to_string(),
                },
            )
            .unwrap();

        assert_eq!(user_liquidation_threshold.u128(), 8992555); // 89.92555%

        let user_utilization_rate: Uint128 = app
            .wrap()
            .query_wasm_smart(
                addr.clone(),
                &QueryMsg::GetUserUtilizationRate {
                    address: "user".to_string(),
                },
            )
            .unwrap();

        assert_eq!(user_utilization_rate.u128(), 8486352); // 84.86352% < 89.92555%

        app.set_block(BlockInfo {
            height: 542,
            time: Timestamp::from_seconds(now + 2 * YEAR_IN_SECONDS + 10000), // after 2 years
            chain_id: "custom_chain_id".to_string(),
        });

        let available_to_borrow_eth: Uint128 = app
            .wrap()
            .query_wasm_smart(
                addr.clone(),
                &QueryMsg::GetAvailableToBorrow {
                    address: "user".to_string(),
                    denom: "eth".to_string(),
                },
            )
            .unwrap();

        assert_eq!(available_to_borrow_eth.u128(), 0);

        let user_liquidation_threshold: Uint128 = app
            .wrap()
            .query_wasm_smart(
                addr.clone(),
                &QueryMsg::GetUserLiquidationThreshold {
                    address: "user".to_string(),
                },
            )
            .unwrap();

        assert_eq!(user_liquidation_threshold.u128(), 8992676); // 89.92676%

        let user_utilization_rate: Uint128 = app
            .wrap()
            .query_wasm_smart(
                addr.clone(),
                &QueryMsg::GetUserUtilizationRate {
                    address: "user".to_string(),
                },
            )
            .unwrap();

        assert_eq!(user_utilization_rate.u128(), 9366274); // 93.66274% > 89.92676%

        let user_deposit_amount_eth: GetBalanceResponse = app
            .wrap()
            .query_wasm_smart(
                addr.clone(),
                &QueryMsg::GetDeposit {
                    address: "user".to_string(),
                    denom: "eth".to_string(),
                },
            )
            .unwrap();

        let user_deposit_amount_atom: GetBalanceResponse = app
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
            user_deposit_amount_eth.balance.u128(),
            203331286529000814400
        ); // 203.331286529000814400 ETH
        assert_eq!(
            user_deposit_amount_atom.balance.u128(),
            300000000000000000000
        ); // 300 ATOM

        let user_borrow_amount_eth: Uint128 = app
            .wrap()
            .query_wasm_smart(
                addr.clone(),
                &QueryMsg::GetUserBorrowAmountWithInterest {
                    address: "user".to_string(),
                    denom: "eth".to_string(),
                },
            )
            .unwrap();

        assert_eq!(user_borrow_amount_eth.u128(), 191850604584630250327); // 191.850604584630250327 ETH

        app.execute_contract(
            Addr::unchecked("liquidator"),
            addr.clone(),
            &ExecuteMsg::Deposit {},
            &coins(LIQUIDATOR_DEPOSIT_AMOUNT_ETH, "eth"),
        )
        .unwrap();

        let liquidator_deposit_amount_eth: GetBalanceResponse = app
            .wrap()
            .query_wasm_smart(
                addr.clone(),
                &QueryMsg::GetDeposit {
                    address: "liquidator".to_string(),
                    denom: "eth".to_string(),
                },
            )
            .unwrap();

        let liquidator_deposit_amount_atom: GetBalanceResponse = app
            .wrap()
            .query_wasm_smart(
                addr.clone(),
                &QueryMsg::GetDeposit {
                    address: "liquidator".to_string(),
                    denom: "atom".to_string(),
                },
            )
            .unwrap();

        assert_eq!(
            liquidator_deposit_amount_eth.balance.u128(),
            9999999999999999999999
        ); // 9999.999999999999999999 ETH
        assert_eq!(liquidator_deposit_amount_atom.balance.u128(), 0); // 0

        app.execute_contract(
            Addr::unchecked("user"),
            addr.clone(),
            &ExecuteMsg::Liquidation {
                user: "user".to_string(),
            },
            &[],
        )
        .unwrap();

        let user_collateral_usd: Uint128 = app
            .wrap()
            .query_wasm_smart(
                addr.clone(),
                &QueryMsg::GetUserCollateralUsd {
                    address: "user".to_string(),
                },
            )
            .unwrap();

        // after liquidation, all collateral is transferred to the liquidator
        assert_eq!(user_collateral_usd.u128(), 0);

        let user_borrowed_usd: Uint128 = app
            .wrap()
            .query_wasm_smart(
                addr.clone(),
                &QueryMsg::GetUserBorrowedUsd {
                    address: "user".to_string(),
                },
            )
            .unwrap();

        // after liquidation, all borrowings are repaid by the liquidator
        assert_eq!(user_borrowed_usd.u128(), 0);

        let liquidator_deposit_amount_eth: GetBalanceResponse = app
            .wrap()
            .query_wasm_smart(
                addr.clone(),
                &QueryMsg::GetDeposit {
                    address: "liquidator".to_string(),
                    denom: "eth".to_string(),
                },
            )
            .unwrap();

        let liquidator_deposit_amount_atom: GetBalanceResponse = app
            .wrap()
            .query_wasm_smart(
                addr.clone(),
                &QueryMsg::GetDeposit {
                    address: "liquidator".to_string(),
                    denom: "atom".to_string(),
                },
            )
            .unwrap();

        assert_eq!(
            liquidator_deposit_amount_eth.balance.u128(),
            10008510511955314159271
        ); // 10008.510511955314159271 ETH
        assert_eq!(
            liquidator_deposit_amount_atom.balance.u128(),
            300000000000000000000
        ); // 300 ATOM
    }
}
