#[cfg(test)]
mod tests {
    use crate::utils::success_deposit_as_collateral_of_diff_token_with_prices;
    use cosmwasm_std::{Addr, Uint128};
    use cw_multi_test::Executor;
    use lending::msg::{ExecuteMsg, QueryMsg};

    #[test]
    fn test_get_available_to_redeem() {
        // contract reserves: 1000 ETH and 1000 ATOM
        // user deposited 200 ETH and 300 ATOM
        // LIQUIDATION_THRESHOLD_ETH = 90%
        // LIQUIDATION_THRESHOLD_ATOM = 80%
        let (mut app, lending_contract_addr, _collateral_contract_addr) =
            success_deposit_as_collateral_of_diff_token_with_prices();

        const TOKENS_DECIMALS: u32 = 18;
        const BORROW_AMOUNT_ATOM: u128 = 1000 * 10u128.pow(TOKENS_DECIMALS); // 1000 ATOM

        let available_to_redeem_eth: Uint128 = app
            .wrap()
            .query_wasm_smart(
                lending_contract_addr.clone(),
                &QueryMsg::GetAvailableToRedeem {
                    address: "user".to_string(),
                    denom: "eth".to_string(),
                },
            )
            .unwrap();

        let available_to_redeem_atom: Uint128 = app
            .wrap()
            .query_wasm_smart(
                lending_contract_addr.clone(),
                &QueryMsg::GetAvailableToRedeem {
                    address: "user".to_string(),
                    denom: "atom".to_string(),
                },
            )
            .unwrap();

        // user hasn't borrowed anything yet
        assert_eq!(available_to_redeem_eth.u128(), 200000000000000000000); // 200 ETH == 400_000$
        assert_eq!(available_to_redeem_atom.u128(), 300000000000000000000); // 300 ATOM == 3000$

        app.execute_contract(
            Addr::unchecked("user"),
            lending_contract_addr.clone(),
            &ExecuteMsg::Borrow {
                denom: "atom".to_string(),
                amount: Uint128::from(BORROW_AMOUNT_ATOM),
            },
            &[],
        )
        .unwrap();

        let user_collateral_usd: Uint128 = app
            .wrap()
            .query_wasm_smart(
                lending_contract_addr.clone(),
                &QueryMsg::GetUserCollateralUsd {
                    address: "user".to_string(),
                },
            )
            .unwrap();

        assert_eq!(
            user_collateral_usd.u128(),
            40300000000000 // 200 ETH * 2000$ + 300 ATOM * 10$ = 403_000$
        );

        // user_liquidation_threshold =
        // (deposit_eth * LIQUIDATION_THRESHOLD_ETH * price_eth
        //    + deposit_atom * LIQUIDATION_THRESHOLD_ATOM * price_atom) / user_collateral_usd =
        // (200 ETH * 0.9 * 2000 + 300 ATOM * 0.8 * 10) / 403_000$ = 362_400$ / 403_000$ ~= 89.92555%
        let user_liquidation_threshold: Uint128 = app
            .wrap()
            .query_wasm_smart(
                lending_contract_addr.clone(),
                &QueryMsg::GetUserLiquidationThreshold {
                    address: "user".to_string(),
                },
            )
            .unwrap();

        assert_eq!(
            user_liquidation_threshold.u128(),
            8992555 // 89.92555%
        );

        let available_to_redeem_eth: Uint128 = app
            .wrap()
            .query_wasm_smart(
                lending_contract_addr.clone(),
                &QueryMsg::GetAvailableToRedeem {
                    address: "user".to_string(),
                    denom: "eth".to_string(),
                },
            )
            .unwrap();

        let available_to_redeem_atom: Uint128 = app
            .wrap()
            .query_wasm_smart(
                lending_contract_addr.clone(),
                &QueryMsg::GetAvailableToRedeem {
                    address: "user".to_string(),
                    denom: "atom".to_string(),
                },
            )
            .unwrap();

        // required_collateral_balance_usd = BORROW_AMOUNT_ATOM * PRICE / user_liquidation_threshold =
        // 1000 ATOM * 10$ / 0.8992555 ~= 11120.31007872$
        // user_collateral_usd - required_collateral_balance_usd = 403_000$ - 11120.31007872$ = 391879.68992128$
        assert_eq!(available_to_redeem_eth.u128(), 195939844960640000000); // 195.93984496064 ETH ~= 391879.68992128$
        assert_eq!(available_to_redeem_atom.u128(), 300000000000000000000); // 300 ATOM == 3000$
    }
}
