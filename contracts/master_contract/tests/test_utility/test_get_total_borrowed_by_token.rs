#[cfg(test)]
mod tests {
    use crate::utils::success_deposit_as_collateral_of_diff_token_with_prices;
    use cosmwasm_std::{
        Addr,
        Uint128
    };
    use cw_multi_test::Executor;
    use master_contract::msg::{
        ExecuteMsg,
        QueryMsg
    };

    #[test]
    fn test_get_total_borrowed_by_token() {
        // contract reserves: 1000 ETH and 1000 ATOM
        // user deposited 200 ETH and 300 ATOM
        let (mut app, addr) = success_deposit_as_collateral_of_diff_token_with_prices();

        const TOKENS_DECIMALS: u32 = 18;
        const BORROW_AMOUNT_ETH: u128 = 10 * 10u128.pow(TOKENS_DECIMALS); // 10 ETH
        const BORROW_AMOUNT_ATOM: u128 = 200 * 10u128.pow(TOKENS_DECIMALS); // 200 ATOM

        let total_borrowed_by_token_eth: Uint128 = app
            .wrap()
            .query_wasm_smart(
                addr.clone(),
                &QueryMsg::GetTotalBorrowedByToken {
                    denom: "eth".to_string(),
                },
            )
            .unwrap();

        let total_borrowed_by_token_atom: Uint128 = app
            .wrap()
            .query_wasm_smart(
                addr.clone(),
                &QueryMsg::GetTotalBorrowedByToken {
                    denom: "atom".to_string(),
                },
            )
            .unwrap();

        // no one has borrowed anything yet
        assert_eq!(total_borrowed_by_token_eth.u128(), 0);
        assert_eq!(total_borrowed_by_token_atom.u128(), 0);

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

        let total_borrowed_by_token_eth: Uint128 = app
            .wrap()
            .query_wasm_smart(
                addr.clone(),
                &QueryMsg::GetTotalBorrowedByToken {
                    denom: "eth".to_string(),
                },
            )
            .unwrap();

        let total_borrowed_by_token_atom: Uint128 = app
            .wrap()
            .query_wasm_smart(
                addr.clone(),
                &QueryMsg::GetTotalBorrowedByToken {
                    denom: "atom".to_string(),
                },
            )
            .unwrap();

        assert_eq!(total_borrowed_by_token_eth.u128(), BORROW_AMOUNT_ETH);
        assert_eq!(total_borrowed_by_token_atom.u128(), BORROW_AMOUNT_ATOM);

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

        let total_borrowed_by_token_eth: Uint128 = app
            .wrap()
            .query_wasm_smart(
                addr.clone(),
                &QueryMsg::GetTotalBorrowedByToken {
                    denom: "eth".to_string(),
                },
            )
            .unwrap();

        let total_borrowed_by_token_atom: Uint128 = app
            .wrap()
            .query_wasm_smart(
                addr.clone(),
                &QueryMsg::GetTotalBorrowedByToken {
                    denom: "atom".to_string(),
                },
            )
            .unwrap();

        assert_eq!(total_borrowed_by_token_eth.u128(), 2*BORROW_AMOUNT_ETH);
        assert_eq!(total_borrowed_by_token_atom.u128(), 2*BORROW_AMOUNT_ATOM);
    }
}
