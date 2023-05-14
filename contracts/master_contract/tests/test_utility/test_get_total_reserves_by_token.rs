#[cfg(test)]
mod tests {
    use crate::utils::success_deposit_as_collateral_of_diff_token_with_prices;
    use cosmwasm_std::{
        Addr,
        Uint128,
        coins,
        BlockInfo,
        Timestamp
    };
    use cw_multi_test::Executor;
    use master_contract::msg::{
        ExecuteMsg,
        QueryMsg
    };
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn test_get_total_reserves_by_token() {
        // contract reserves: 1000 ETH and 1000 ATOM
        // user deposited 200 ETH and 300 ATOM
        let (mut app, addr) = success_deposit_as_collateral_of_diff_token_with_prices();

        const TOKENS_DECIMALS: u32 = 18;
        const BORROW_AMOUNT_ETH: u128 = 10 * 10u128.pow(TOKENS_DECIMALS); // 10 ETH
        const BORROW_AMOUNT_ATOM: u128 = 200 * 10u128.pow(TOKENS_DECIMALS); // 200 ATOM
        const DEPOSIT_AMOUNT_ETH: u128 = 30 * 10u128.pow(TOKENS_DECIMALS); // 30 ETH
        const DEPOSIT_AMOUNT_ATOM: u128 = 400 * 10u128.pow(TOKENS_DECIMALS); // 400 ATOM

        const YEAR_IN_SECONDS: u64 = 31536000;

        let total_reserves_by_token_eth: Uint128 = app
            .wrap()
            .query_wasm_smart(
                addr.clone(),
                &QueryMsg::GetTotalReservesByToken {
                    denom: "eth".to_string(),
                },
            )
            .unwrap();

        let total_reserves_by_token_atom: Uint128 = app
            .wrap()
            .query_wasm_smart(
                addr.clone(),
                &QueryMsg::GetTotalReservesByToken {
                    denom: "atom".to_string(),
                },
            )
            .unwrap();

        assert_eq!(total_reserves_by_token_eth.u128(), 1200000000000000000000); // 1000 ETH + 200 ETH = 1200 ETH
        assert_eq!(total_reserves_by_token_atom.u128(), 1300000000000000000000); // 1000 ATOM + 300 ATOM = 1300 ATOM

        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();

        app.set_block(BlockInfo {
            height: 0,
            time: Timestamp::from_seconds(now),
            chain_id: "custom_chain_id".to_string(),
        });

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

        let total_reserves_by_token_eth: Uint128 = app
            .wrap()
            .query_wasm_smart(
                addr.clone(),
                &QueryMsg::GetTotalReservesByToken {
                    denom: "eth".to_string(),
                },
            )
            .unwrap();

        let total_reserves_by_token_atom: Uint128 = app
            .wrap()
            .query_wasm_smart(
                addr.clone(),
                &QueryMsg::GetTotalReservesByToken {
                    denom: "atom".to_string(),
                },
            )
            .unwrap();

        // the reserve includes all borrowings, so taking a new borrowing does not affect the reserve
        assert_eq!(total_reserves_by_token_eth.u128(), 1200000000000000000000); // 1000 ETH + 200 ETH = 1200 ETH
        assert_eq!(total_reserves_by_token_atom.u128(), 1300000000000000000000); // 1000 ATOM + 300 ATOM = 1300 ATOM

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

        let total_reserves_by_token_eth: Uint128 = app
            .wrap()
            .query_wasm_smart(
                addr.clone(),
                &QueryMsg::GetTotalReservesByToken {
                    denom: "eth".to_string(),
                },
            )
            .unwrap();

        let total_reserves_by_token_atom: Uint128 = app
            .wrap()
            .query_wasm_smart(
                addr.clone(),
                &QueryMsg::GetTotalReservesByToken {
                    denom: "atom".to_string(),
                },
            )
            .unwrap();

        assert_eq!(total_reserves_by_token_eth.u128(), 1230000000000000000000); // 1000 ETH + 200 ETH + 30 ETH = 1230 ETH
        assert_eq!(total_reserves_by_token_atom.u128(), 1700000000000000000000); // 1000 ATOM + 300 ATOM + 400 ATOM = 1700 ATOM

        app.set_block(BlockInfo {
            height: 0,
            time: Timestamp::from_seconds(now + YEAR_IN_SECONDS),
            chain_id: "custom_chain_id".to_string(),
        });

        let total_reserves_by_token_eth: Uint128 = app
            .wrap()
            .query_wasm_smart(
                addr.clone(),
                &QueryMsg::GetTotalReservesByToken {
                    denom: "eth".to_string(),
                },
            )
            .unwrap();

        let total_reserves_by_token_atom: Uint128 = app
            .wrap()
            .query_wasm_smart(
                addr.clone(),
                &QueryMsg::GetTotalReservesByToken {
                    denom: "atom".to_string(),
                },
            )
            .unwrap();

        // interest accrued for the year and included in total reserves
        // 1230 ETH + 10 ETH * 5% borrow APY = 1230 ETH + 0.5 ETH = 1230.5 ETH
        assert_eq!(total_reserves_by_token_eth.u128(), 1230500000000000000000);
        // 1700 ATOM + 200 ATOM * 5% borrow APY = 1700 ATOM + 10 ATOM = 1710 ATOM
        assert_eq!(total_reserves_by_token_atom.u128(), 1710000000000000000000);
    }
}
