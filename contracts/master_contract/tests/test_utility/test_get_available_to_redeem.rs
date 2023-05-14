#[cfg(test)]
mod tests {
    use crate::utils::success_deposit_as_collateral_of_diff_token_with_prices;
    use cosmwasm_std::{Addr, Uint128};
    use cw_multi_test::Executor;
    use master_contract::msg::{ExecuteMsg, QueryMsg};

    #[test]
    fn test_get_available_to_redeem() {
        // contract reserves: 1000 ETH and 1000 ATOM
        // user deposited 200 ETH and 300 ATOM
        let (mut app, addr) = success_deposit_as_collateral_of_diff_token_with_prices();

        const TOKENS_DECIMALS: u32 = 18;
        const BORROW_AMOUNT_ATOM: u128 = 1000 * 10u128.pow(TOKENS_DECIMALS); // 1000 ATOM

        let user_deposited_usd: Uint128 = app
            .wrap()
            .query_wasm_smart(
                addr.clone(),
                &QueryMsg::GetUserDepositedUsd {
                    address: "user".to_string(),
                },
            )
            .unwrap();

        assert_eq!(
            user_deposited_usd.u128(),
            40300000000000 // 300 ATOM * 10$ + 200 ETH * 2000$ = 403000$
        );

        let available_to_redeem_eth: Uint128 = app
            .wrap()
            .query_wasm_smart(
                addr.clone(),
                &QueryMsg::GetAvailableToRedeem {
                    address: "user".to_string(),
                    denom: "eth".to_string(),
                },
            )
            .unwrap();

        let available_to_redeem_atom: Uint128 = app
            .wrap()
            .query_wasm_smart(
                addr.clone(),
                &QueryMsg::GetAvailableToRedeem {
                    address: "user".to_string(),
                    denom: "atom".to_string(),
                },
            )
            .unwrap();

        assert_eq!(available_to_redeem_eth.u128(), 200000000000000000000); // 200 ETH == 400000$
        assert_eq!(available_to_redeem_atom.u128(), 300000000000000000000); // 300 ATOM == 3000$

        app.execute_contract(
            Addr::unchecked("user"),
            addr.clone(),
            &ExecuteMsg::Borrow {
                denom: "atom".to_string(),
                amount: Uint128::from(BORROW_AMOUNT_ATOM),
            },
            &[],
        )
        .unwrap();

        let available_to_redeem_eth: Uint128 = app
            .wrap()
            .query_wasm_smart(
                addr.clone(),
                &QueryMsg::GetAvailableToRedeem {
                    address: "user".to_string(),
                    denom: "eth".to_string(),
                },
            )
            .unwrap();

        let available_to_redeem_atom: Uint128 = app
            .wrap()
            .query_wasm_smart(
                addr.clone(),
                &QueryMsg::GetAvailableToRedeem {
                    address: "user".to_string(),
                    denom: "atom".to_string(),
                },
            )
            .unwrap();

        // required_collateral_balance_usd = BORROW_AMOUNT_ATOM * PRICE / 0.8 = 1000 ATOM * 10$ / 0.8 = 12500$
        // user_deposited_usd - required_collateral_balance_usd = 403000$ - 12500$ = 390500$
        assert_eq!(available_to_redeem_eth.u128(), 195250000000000000000); // 195.25 ETH == 390500$
        assert_eq!(available_to_redeem_atom.u128(), 300000000000000000000); // 300 ATOM == 3000$
    }
}
