#[cfg(test)]
mod tests {
    use crate::utils::success_deposit_of_diff_token_with_prices;
    use cosmwasm_std::{coins, Addr, Uint128};
    use cw_multi_test::Executor;
    use lending::msg::{ExecuteMsg, QueryMsg};

    #[test]
    fn test_get_user_collateral_usd() {
        const ATOM_DECIMALS: u32 = 18;
        const DEPOSIT_AMOUNT_ATOM: u128 = 500u128 * 10u128.pow(ATOM_DECIMALS); // 500 ATOM

        // contract reserves: 1000 ETH and 1000 ATOM
        // user deposited 200 ETH and 300 ATOM
        let (mut app, addr) = success_deposit_of_diff_token_with_prices();

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

        let user_collateral_usd: Uint128 = app
            .wrap()
            .query_wasm_smart(
                addr.clone(),
                &QueryMsg::GetUserCollateralUsd {
                    address: "user".to_string(),
                },
            )
            .unwrap();

        assert_eq!(user_collateral_usd.u128(), 0);

        app.execute_contract(
            Addr::unchecked("user"),
            addr.clone(),
            &ExecuteMsg::ToggleCollateralSetting {
                denom: "eth".to_string(),
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

        assert_eq!(user_collateral_usd.u128(), 40000000000000); // 200 ETH deposit == 400_000$

        app.execute_contract(
            Addr::unchecked("user"),
            addr.clone(),
            &ExecuteMsg::Deposit {},
            &coins(DEPOSIT_AMOUNT_ATOM, "atom"),
        )
        .unwrap();

        let user_atom_deposit_as_collateral: bool = app
            .wrap()
            .query_wasm_smart(
                addr.clone(),
                &QueryMsg::UserDepositAsCollateral {
                    address: "user".to_string(),
                    denom: "atom".to_string(),
                },
            )
            .unwrap();

        assert_eq!(user_atom_deposit_as_collateral, false);

        let user_collateral_usd: Uint128 = app
            .wrap()
            .query_wasm_smart(
                addr.clone(),
                &QueryMsg::GetUserCollateralUsd {
                    address: "user".to_string(),
                },
            )
            .unwrap();

        // ATOM user deposits do not count because user_atom_deposit_as_collateral == false
        assert_eq!(user_collateral_usd.u128(), 40000000000000); // 200 ETH deposit + 0 ATOM == 400_000$

        app.execute_contract(
            Addr::unchecked("user"),
            addr.clone(),
            &ExecuteMsg::ToggleCollateralSetting {
                denom: "atom".to_string(),
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

        assert_eq!(user_collateral_usd.u128(), 40800000000000); // 200 ETH deposit * 2000 + 800 ATOM * 10 = 408_000$
    }
}
