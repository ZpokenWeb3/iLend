#[cfg(test)]
mod tests {
    //     use super::*;
    use crate::utils::success_deposit_of_diff_token_with_prices;
    //     use cosmwasm_schema::serde::__private::de::IdentifierDeserializer;
    use cosmwasm_std::{
        Addr,
        Uint128,
        coins
    };
    use cw_multi_test::Executor;
    use master_contract::msg::{
        ExecuteMsg,
        QueryMsg,
        GetUserDepositedUsdResponse,
        GetBalanceResponse
    };

    #[test]
    fn test_user_deposit_as_collateral() {
        // having 500 deposited we want to redeem SECOND_DEPOSIT_AMOUNT
        // so that FIRST_DEPOSIT_AMOUNT is remaining
        let (app, addr) = success_deposit_of_diff_token_with_prices();

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

        assert_eq!(user_eth_deposit_as_collateral, false);
        assert_eq!(user_atom_deposit_as_collateral, false);
    }

    #[test]
    fn test_toggle_collateral_setting() {
        // having 500 deposited we want to redeem SECOND_DEPOSIT_AMOUNT
        // so that FIRST_DEPOSIT_AMOUNT is remaining
        let (mut app, addr) = success_deposit_of_diff_token_with_prices();

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

        assert_eq!(user_eth_deposit_as_collateral, true);
        assert_eq!(user_atom_deposit_as_collateral, false);
    }

    #[test]
    #[should_panic(expected = "The collateral has already using to collateralise the borrowing. Not enough available balance")]
    fn test_fail_toggle_collateral_setting_when_not_enough_available_balance() {
        const ATOM_DECIMALS: u32 = 18;
        const ETH_DECIMALS: u32 = 18;

        const DEPOSIT_AMOUNT_ATOM: u128 = 500000u128 * 10u128.pow(ATOM_DECIMALS); // 500000 ATOM
        const BORROW_AMOUNT_ETH: u128 = 160u128 * 10u128.pow(ETH_DECIMALS); // 160 ETH

        // contract reserves: 1000 ETH and 1000 ATOM
        // user deposited 200 ETH and 300 ATOM
        let (mut app, addr) = success_deposit_of_diff_token_with_prices();

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
            &ExecuteMsg::Deposit {},
            &coins(DEPOSIT_AMOUNT_ATOM, "atom"),
        )
        .unwrap();

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

        assert_eq!(user_deposited_balance_atom.balance.u128(), 500300000000000000000000); // 500300 ATOM

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

        assert_eq!(user_deposited_balance_eth.balance.u128(), 200000000000000000000); // 200 ETH

        let sum_collateral_balance_usd: GetUserDepositedUsdResponse = app
            .wrap()
            .query_wasm_smart(
                addr.clone(),
                &QueryMsg::GetUserDepositedUsd {
                    address: "user".to_string(),
                },
            )
            .unwrap();

        assert_eq!(sum_collateral_balance_usd.user_deposited_usd.u128(), 540300000000000); // 500300 ATOM * 10 + 200 ETH * 2000 = 5003000$ + 400000$ = 5403000$

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

        // Only the ether deposit counts as collateral => 200 ETH * 0.8 = 160 ETH
        assert_eq!(available_to_borrow_eth.u128(), 160000000000000000000); // 160 ETH == 320_000$

        app.execute_contract(
            Addr::unchecked("user"),
            addr.clone(),
            &ExecuteMsg::Borrow {
                denom: "eth".to_string(),
                amount: Uint128::from(BORROW_AMOUNT_ETH),
            },
            &[],
        ).unwrap();

        app.execute_contract(
            Addr::unchecked("user"),
            addr.clone(),
            &ExecuteMsg::ToggleCollateralSetting {
                denom: "eth".to_string(),
            },
            &[],
        ).unwrap();
    }
}
