#[cfg(test)]
mod tests {
    use crate::utils::success_deposit_as_collateral_of_diff_token_with_prices;
    use cosmwasm_std::{coins, Addr, Uint128};
    use cw_multi_test::Executor;
    use lending::msg::{ExecuteMsg, QueryMsg};

    #[test]
    fn test_get_user_liquidation_threshold() {
        const TOKENS_DECIMALS: u32 = 18;
        const DEPOSIT_AMOUNT_ETH: u128 = 30 * 10u128.pow(TOKENS_DECIMALS); // 30 ETH
        const DEPOSIT_AMOUNT_ATOM: u128 = 400 * 10u128.pow(TOKENS_DECIMALS); // 400 ATOM

        // contract reserves: 1000 ETH and 1000 ATOM
        // user deposited 200 ETH and 300 ATOM
        // LIQUIDATION_THRESHOLD_ETH = 90%
        // LIQUIDATION_THRESHOLD_ATOM = 80%
        let (mut app, addr) = success_deposit_as_collateral_of_diff_token_with_prices();

        let user_collateral_usd: Uint128 = app
            .wrap()
            .query_wasm_smart(
                addr.clone(),
                &QueryMsg::GetUserCollateralUsd {
                    address: "user".to_string(),
                },
            )
            .unwrap();

        assert_eq!(
            user_collateral_usd.u128(),
            40300000000000 // 200 ETH * 2000$ + 300 ATOM * 10$ = 403_000$
        );

        let user_liquidation_threshold: Uint128 = app
            .wrap()
            .query_wasm_smart(
                addr.clone(),
                &QueryMsg::GetUserLiquidationThreshold {
                    address: "user".to_string(),
                },
            )
            .unwrap();

        // user_liquidation_threshold =
        // (deposit_eth * LIQUIDATION_THRESHOLD_ETH * price_eth
        //    + deposit_atom * LIQUIDATION_THRESHOLD_ATOM * price_atom) / user_collateral_usd =
        // (200 ETH * 0.9 * 2000 + 300 ATOM * 0.8 * 10) / 403_000$ = 362_400$ / 403_000$ ~= 89.92555%
        assert_eq!(user_liquidation_threshold.u128(), 8992555);

        app.execute_contract(
            Addr::unchecked("user"),
            addr.clone(),
            &ExecuteMsg::Deposit {},
            &coins(DEPOSIT_AMOUNT_ETH, "eth"),
        )
        .unwrap();

        app.execute_contract(
            Addr::unchecked("user"),
            addr.clone(),
            &ExecuteMsg::Deposit {},
            &coins(DEPOSIT_AMOUNT_ATOM, "atom"),
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

        assert_eq!(
            user_collateral_usd.u128(),
            46700000000000 // 230 ETH * 2000$ + 700 ATOM * 10$ = 467_000$
        );

        let user_liquidation_threshold: Uint128 = app
            .wrap()
            .query_wasm_smart(
                addr.clone(),
                &QueryMsg::GetUserLiquidationThreshold {
                    address: "user".to_string(),
                },
            )
            .unwrap();

        // user_liquidation_threshold =
        // (deposit_eth * LIQUIDATION_THRESHOLD_ETH * price_eth
        //    + deposit_atom * LIQUIDATION_THRESHOLD_ATOM * price_atom) / user_collateral_usd =
        // (230 ETH * 0.9 * 2000 + 700 ATOM * 0.8 * 10) / 467_000$ = 419_600$ / 467_000$ ~= 89.85010%
        assert_eq!(user_liquidation_threshold.u128(), 8985010); // 89.85010%
    }
}
