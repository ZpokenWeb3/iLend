#[cfg(test)]
mod tests {
    use crate::utils::success_deposit_as_collateral_of_diff_token_with_prices;
    use cosmwasm_std::{
        Addr,
        Uint128,
        coins
    };
    use cw_multi_test::Executor;
    use master_contract::msg::{
        ExecuteMsg,
        QueryMsg
    };

    #[test]
    fn test_get_available_to_borrow() {
        // contract reserves: 1000 ETH and 1000 ATOM
        // user deposited 200 ETH and 300 ATOM
        let (mut app, addr) = success_deposit_as_collateral_of_diff_token_with_prices();

        const TOKENS_DECIMALS: u32 = 18;
        const DEPOSIT_AMOUNT_ETH: u128 = 30 * 10u128.pow(TOKENS_DECIMALS); // 30 ETH
        const DEPOSIT_AMOUNT_ATOM: u128 = 400 * 10u128.pow(TOKENS_DECIMALS); // 400 ATOM
        const BORROW_AMOUNT_ETH: u128 = 100 * 10u128.pow(TOKENS_DECIMALS); // 100 ETH
        const BORROW_AMOUNT_ATOM: u128 = 1000 * 10u128.pow(TOKENS_DECIMALS); // 1000 ATOM

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
            40300000000000 // 200 ETH * 2000$ + 300 ATOM * 10$ = 403000$
        );

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

        let available_to_borrow_atom: Uint128 = app
            .wrap()
            .query_wasm_smart(
                addr.clone(),
                &QueryMsg::GetAvailableToBorrow {
                    address: "user".to_string(),
                    denom: "atom".to_string(),
                },
            )
            .unwrap();

        // user_collateral_usd * 0.8 / price = 403000$ * 0.8 / price = 322400$ / price
        assert_eq!(available_to_borrow_eth.u128(), 161200000000000000000); // 322400$ / 2000 == 161.2 ETH
        // the amount of user deposits allow the user to borrow 322400$ / 10 == 32240 ATOM
        // but the contract has only 1300 ATOM liquidity
        assert_eq!(available_to_borrow_atom.u128(), 1300000000000000000000); // 1300 ATOM

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

        let user_borrowed_usd: Uint128 = app
            .wrap()
            .query_wasm_smart(
                addr.clone(),
                &QueryMsg::GetUserBorrowedUsd {
                    address: "user".to_string(),
                },
            )
            .unwrap();

        assert_eq!(
            user_borrowed_usd.u128(),
            21000000000000 // 100 ETH * 2000$ + 1000 ATOM * 10$ = 210000$
        );

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

        let available_to_borrow_atom: Uint128 = app
            .wrap()
            .query_wasm_smart(
                addr.clone(),
                &QueryMsg::GetAvailableToBorrow {
                    address: "user".to_string(),
                    denom: "atom".to_string(),
                },
            )
            .unwrap();

        // (user_deposited_usd * 0.8 - user_borrowed_usd) / price = (403000$ * 0.8 - 210000$) / price = 112400$ / price
        assert_eq!(available_to_borrow_eth.u128(), 56200000000000000000); // 112400$ / 2000 == 56.2 ETH
        // the amount of user deposits allow the user to borrow 112400$ / 10 == 11240 ATOM
        // but the contract has only 1300 ATOM - 1000 ATOM = 300 ATOM liquidity
        assert_eq!(available_to_borrow_atom.u128(), 300000000000000000000); // 300 ATOM

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
            46700000000000 // 230 ETH * 2000$ + 700 ATOM * 10$ = 460000$ + 7000$ = 467000$
        );

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

        let available_to_borrow_atom: Uint128 = app
            .wrap()
            .query_wasm_smart(
                addr.clone(),
                &QueryMsg::GetAvailableToBorrow {
                    address: "user".to_string(),
                    denom: "atom".to_string(),
                },
            )
            .unwrap();

        // (user_deposited_usd * 0.8 - user_borrowed_usd) / price = (467000$ * 0.8 - 210000$) / price = 163600$ / price
        assert_eq!(available_to_borrow_eth.u128(), 81800000000000000000); // 163600$ / 2000 == 81.8 ETH
        // the amount of user deposits allow the user to borrow 163600$ / 10 == 16360 ATOM
        // but the contract has only 1300 ATOM - 1000 ATOM + 400 ATOM = 700 ATOM liquidity
        assert_eq!(available_to_borrow_atom.u128(), 700000000000000000000); // 700 ATOM
    }
}
