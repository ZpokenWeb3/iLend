#[cfg(test)]
mod tests {
    use crate::utils::success_deposit_as_collateral_of_diff_token_with_prices;
    use cosmwasm_std::{coins, Addr, Uint128};
    use cw_multi_test::Executor;
    use master_contract::msg::{ExecuteMsg, QueryMsg};

    #[test]
    fn test_get_available_to_borrow() {
        // contract reserves: 1000 ETH and 1000 ATOM
        // user deposited 200 ETH and 300 ATOM
        // ltv_eth = 0.85
        // ltv_atom = 0.75
        let (mut app, addr) = success_deposit_as_collateral_of_diff_token_with_prices();

        const TOKENS_DECIMALS: u32 = 18;
        const DEPOSIT_AMOUNT_ETH: u128 = 30 * 10u128.pow(TOKENS_DECIMALS); // 30 ETH
        const DEPOSIT_AMOUNT_ATOM: u128 = 400 * 10u128.pow(TOKENS_DECIMALS); // 400 ATOM
        const BORROW_AMOUNT_ETH: u128 = 100 * 10u128.pow(TOKENS_DECIMALS); // 100 ETH
        const BORROW_AMOUNT_ATOM: u128 = 1000 * 10u128.pow(TOKENS_DECIMALS); // 1000 ATOM

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

        // (deposit_eth * ltv_eth * price_eth + deposit_atom * ltv_atom * price_atom) / price = 
        // (200 * 0.85 * 2000 + 300 * 0.75 * 10) / price = 342250$ / price
        assert_eq!(available_to_borrow_eth.u128(), 171125000000000000000); // 342250$ / 2000 == 171.125 ETH

        // the amount of user deposits allow the user to borrow 342250$ / 10 == 34225 ATOM
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

        // (deposit_eth * ltv_eth * price_eth + deposit_atom * ltv_atom * price_atom - user_borrowed_usd) / price = 
        // (200 * 0.85 * 2000 + 300 * 0.75 * 10 - 210000$) / price = 132250$ / price
        assert_eq!(available_to_borrow_eth.u128(), 66125000000000000000); // 132250$ / 2000 == 66.125 ETH

        // the amount of user deposits allow the user to borrow 132250$ / 10 == 13225 ATOM
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

        // (deposit_eth * ltv_eth * price_eth + deposit_atom * ltv_atom * price_atom - user_borrowed_usd) / price = 
        // (230 * 0.85 * 2000 + 700 * 0.75 * 10 - 210000$) / price = 186250$ / price
        assert_eq!(available_to_borrow_eth.u128(), 93125000000000000000); // 186250$ / 2000 == 93.125 ETH

        // the amount of user deposits allow the user to borrow 186250$ / 10 == 18625 ATOM
        // but the contract has only 1300 ATOM - 1000 ATOM + 400 ATOM = 700 ATOM liquidity
        assert_eq!(available_to_borrow_atom.u128(), 700000000000000000000); // 700 ATOM
    }
}
