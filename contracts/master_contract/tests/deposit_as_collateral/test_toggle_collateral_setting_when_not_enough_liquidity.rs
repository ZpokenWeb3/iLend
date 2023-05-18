#[cfg(test)]
mod tests {
    use crate::utils::success_deposit_of_diff_token_with_prices;
    use cosmwasm_std::{coins, Addr, Uint128};
    use cw_multi_test::Executor;
    use master_contract::msg::{ExecuteMsg, GetBalanceResponse, QueryMsg};

    #[test]
    fn test_toggle_collateral_setting_when_not_enough_liquidity() {
        const ATOM_DECIMALS: u32 = 18;
        const ETH_DECIMALS: u32 = 18;

        const OWNER_DEPOSIT_AMOUNT_ATOM: u128 = 500_000 * 10u128.pow(ATOM_DECIMALS); // 500_000 ATOM
        const USER_DEPOSIT_AMOUNT_ATOM: u128 = 200_000 * 10u128.pow(ATOM_DECIMALS); // 200_000 ATOM

        const OWNER_BORROW_AMOUNT_ETH: u128 = 1100 * 10u128.pow(ETH_DECIMALS); // 1100 ETH
        const USER_BORROW_AMOUNT_ETH: u128 = 100 * 10u128.pow(ETH_DECIMALS); // 100 ETH

        // contract reserves: 1000 ETH and 1000 ATOM
        // user deposited 200 ETH and 300 ATOM
        let (mut app, addr) = success_deposit_of_diff_token_with_prices();

        let available_liquidity_by_token_eth: Uint128 = app
            .wrap()
            .query_wasm_smart(
                addr.clone(),
                &QueryMsg::GetAvailableLiquidityByToken {
                    denom: "eth".to_string(),
                },
            )
            .unwrap();

        assert_eq!(
            available_liquidity_by_token_eth.u128(),
            1200000000000000000000
        ); // 1000 ETH + 200 ETH = 1200 ETH

        app.execute_contract(
            Addr::unchecked("owner"),
            addr.clone(),
            &ExecuteMsg::ToggleCollateralSetting {
                denom: "atom".to_string(),
            },
            &[],
        )
        .unwrap();

        app.execute_contract(
            Addr::unchecked("owner"),
            addr.clone(),
            &ExecuteMsg::Deposit {},
            &coins(OWNER_DEPOSIT_AMOUNT_ATOM, "atom"), // 500_000 ATOM
        )
        .unwrap();

        app.execute_contract(
            Addr::unchecked("owner"),
            addr.clone(),
            &ExecuteMsg::Borrow {
                denom: "eth".to_string(),
                amount: Uint128::from(OWNER_BORROW_AMOUNT_ETH), // 1100 ETH
            },
            &[],
        )
        .unwrap();

        let available_liquidity_by_token_eth: Uint128 = app
            .wrap()
            .query_wasm_smart(
                addr.clone(),
                &QueryMsg::GetAvailableLiquidityByToken {
                    denom: "eth".to_string(),
                },
            )
            .unwrap();

        assert_eq!(
            available_liquidity_by_token_eth.u128(),
            100000000000000000000
        ); // 1000 ETH + 200 ETH - 1100 ETH = 100 ETH

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
            Addr::unchecked("user"),
            addr.clone(),
            &ExecuteMsg::ToggleCollateralSetting {
                denom: "atom".to_string(),
            },
            &[],
        )
        .unwrap();

        app.execute_contract(
            Addr::unchecked("user"),
            addr.clone(),
            &ExecuteMsg::Deposit {},
            &coins(USER_DEPOSIT_AMOUNT_ATOM, "atom"), // 200_000 ATOM
        )
        .unwrap();

        app.execute_contract(
            Addr::unchecked("user"),
            addr.clone(),
            &ExecuteMsg::Borrow {
                denom: "eth".to_string(),
                amount: Uint128::from(USER_BORROW_AMOUNT_ETH), // 100 ETH
            },
            &[],
        )
        .unwrap();

        let available_liquidity_by_token_eth: Uint128 = app
            .wrap()
            .query_wasm_smart(
                addr.clone(),
                &QueryMsg::GetAvailableLiquidityByToken {
                    denom: "eth".to_string(),
                },
            )
            .unwrap();

        assert_eq!(
            available_liquidity_by_token_eth.u128(),
            0
        ); // 1000 ETH + 200 ETH - 1100 ETH - 100 ETH = 0

        let user_collateral_usd: Uint128 = app
            .wrap()
            .query_wasm_smart(
                addr.clone(),
                &QueryMsg::GetUserCollateralUsd {
                    address: "user".to_string(),
                },
            )
            .unwrap();

        assert_eq!(user_collateral_usd.u128(), 240300000000000); // 200 ETH * 2000 + (200_000 ATOM + 300 ATOM) * 10 = 2_403_000$

        let user_borrowed_usd: Uint128 = app
            .wrap()
            .query_wasm_smart(
                addr.clone(),
                &QueryMsg::GetUserBorrowedUsd {
                    address: "user".to_string(),
                },
            )
            .unwrap();

        assert_eq!(user_borrowed_usd.u128(), 20000000000000); // 100 ETH * 2000 = 200_000$

        // user toggles collateral setting for ETH deposit so that the ETH deposit is not a collateral
        app.execute_contract(
            Addr::unchecked("user"),
            addr.clone(),
            &ExecuteMsg::ToggleCollateralSetting {
                denom: "eth".to_string(),
            },
            &[],
        )
        .unwrap();

        let user_eth_deposit_as_collateral: bool = app
            .wrap()
            .query_wasm_smart(
                addr.clone(),
                &QueryMsg::UserDepositAsCollateral {
                    address: "user".to_string(),
                    denom: "eth".to_string(),
                },
            )
            .unwrap();

        assert_eq!(user_eth_deposit_as_collateral, false);
    }
}
