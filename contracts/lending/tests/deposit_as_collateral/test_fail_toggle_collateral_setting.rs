#[cfg(test)]
mod tests {
    use crate::utils::success_deposit_of_diff_token_with_prices;
    use cosmwasm_std::{coins, Addr, Uint128};
    use cw_multi_test::Executor;
    use lending::msg::{ExecuteMsg, GetBalanceResponse, QueryMsg};

    #[test]
    #[should_panic(
        expected = "The collateral has already using to collateralise the borrowing. Not enough available balance"
    )]
    fn test_fail_toggle_collateral_setting_when_not_enough_available_balance() {
        const ATOM_DECIMALS: u32 = 18;
        const ETH_DECIMALS: u32 = 18;

        const DEPOSIT_AMOUNT_ATOM: u128 = 500_000 * 10u128.pow(ATOM_DECIMALS); // 500_000 ATOM
        const BORROW_AMOUNT_ETH: u128 = 160 * 10u128.pow(ETH_DECIMALS); // 160 ETH

        // contract reserves: 1000 ETH and 1000 ATOM
        // user deposited 200 ETH and 300 ATOM
        // LTV_ETH = 85%
        let (mut app, lending_contract_addr, _collateral_contract_addr) =
            success_deposit_of_diff_token_with_prices();

        app.execute_contract(
            Addr::unchecked("user"),
            lending_contract_addr.clone(),
            &ExecuteMsg::ToggleCollateralSetting {
                denom: "eth".to_string(),
            },
            &[],
        )
        .unwrap();

        app.execute_contract(
            Addr::unchecked("user"),
            lending_contract_addr.clone(),
            &ExecuteMsg::Deposit {},
            &coins(DEPOSIT_AMOUNT_ATOM, "atom"),
        )
        .unwrap();

        let user_deposited_balance_atom: GetBalanceResponse = app
            .wrap()
            .query_wasm_smart(
                lending_contract_addr.clone(),
                &QueryMsg::GetDeposit {
                    address: "user".to_string(),
                    denom: "atom".to_string(),
                },
            )
            .unwrap();

        assert_eq!(
            user_deposited_balance_atom.balance.u128(),
            500300000000000000000000
        ); // 500000 ATOM + 300 ATOM = 500300 ATOM

        let user_deposited_balance_eth: GetBalanceResponse = app
            .wrap()
            .query_wasm_smart(
                lending_contract_addr.clone(),
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

        let sum_collateral_balance_usd: Uint128 = app
            .wrap()
            .query_wasm_smart(
                lending_contract_addr.clone(),
                &QueryMsg::GetUserDepositedUsd {
                    address: "user".to_string(),
                },
            )
            .unwrap();

        // 500300 ATOM * 10 + 200 ETH * 2000 = 5003000$ + 400000$ = 5403000$
        assert_eq!(sum_collateral_balance_usd.u128(), 540300000000000);

        let available_to_borrow_eth: Uint128 = app
            .wrap()
            .query_wasm_smart(
                lending_contract_addr.clone(),
                &QueryMsg::GetAvailableToBorrow {
                    address: "user".to_string(),
                    denom: "eth".to_string(),
                },
            )
            .unwrap();

        // Only the ETH deposit counts as collateral =>
        // available_to_borrow_eth = user_deposited_balance_eth * LTV_ETH =
        // 200 ETH * 0.85 = 170 ETH
        assert_eq!(available_to_borrow_eth.u128(), 170000000000000000000); // 170 ETH = 340_000$

        app.execute_contract(
            Addr::unchecked("user"),
            lending_contract_addr.clone(),
            &ExecuteMsg::Borrow {
                denom: "eth".to_string(),
                amount: Uint128::from(BORROW_AMOUNT_ETH),
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

        assert_eq!(user_collateral_usd.u128(), 40000000000000); // 200 ETH * 2000 = 400_000$

        let user_borrowed_usd: Uint128 = app
            .wrap()
            .query_wasm_smart(
                lending_contract_addr.clone(),
                &QueryMsg::GetUserBorrowedUsd {
                    address: "user".to_string(),
                },
            )
            .unwrap();

        assert_eq!(user_borrowed_usd.u128(), 32000000000000); // 160 ETH * 2000 = 320_000$

        // toggle attempt unsuccessful since the user has a debt
        // and excluding the ETH deposit from the collateral will result in insufficient collateral.
        // user_collateral_usd - user_eth_deposited_usd < user_borrowed_usd / user_liquidation_threshold
        // 400_000$ - (200 ETH * 2000) = 0
        app.execute_contract(
            Addr::unchecked("user"),
            lending_contract_addr.clone(),
            &ExecuteMsg::ToggleCollateralSetting {
                denom: "eth".to_string(),
            },
            &[],
        )
        .unwrap();
    }
}
