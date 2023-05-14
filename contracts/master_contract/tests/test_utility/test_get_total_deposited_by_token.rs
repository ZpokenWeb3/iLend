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
    fn test_get_total_deposited_by_token() {
        // contract reserves: 1000 ETH and 1000 ATOM
        // user deposited 200 ETH and 300 ATOM
        let (mut app, addr) = success_deposit_as_collateral_of_diff_token_with_prices();

        const TOKENS_DECIMALS: u32 = 18;
        const DEPOSIT_AMOUNT_ETH: u128 = 30 * 10u128.pow(TOKENS_DECIMALS); // 30 ETH
        const DEPOSIT_AMOUNT_ATOM: u128 = 400 * 10u128.pow(TOKENS_DECIMALS); // 400 ATOM
        const BORROW_AMOUNT_ETH: u128 = 100 * 10u128.pow(TOKENS_DECIMALS); // 100 ETH
        const BORROW_AMOUNT_ATOM: u128 = 500 * 10u128.pow(TOKENS_DECIMALS); // 500 ATOM

        const YEAR_IN_SECONDS: u64 = 31536000;

        let total_deposited_by_token_eth: Uint128 = app
            .wrap()
            .query_wasm_smart(
                addr.clone(),
                &QueryMsg::GetTotalDepositedByToken {
                    denom: "eth".to_string(),
                },
            )
            .unwrap();

        let total_deposited_by_token_atom: Uint128 = app
            .wrap()
            .query_wasm_smart(
                addr.clone(),
                &QueryMsg::GetTotalDepositedByToken {
                    denom: "atom".to_string(),
                },
            )
            .unwrap();

        assert_eq!(total_deposited_by_token_eth.u128(), 200000000000000000000); // 200 ETH
        assert_eq!(total_deposited_by_token_atom.u128(), 300000000000000000000); // 300 ATOM

        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();

        app.set_block(BlockInfo {
            height: 0,
            time: Timestamp::from_seconds(now),
            chain_id: "custom_chain_id".to_string(),
        });

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

        let total_deposited_by_token_eth: Uint128 = app
            .wrap()
            .query_wasm_smart(
                addr.clone(),
                &QueryMsg::GetTotalDepositedByToken {
                    denom: "eth".to_string(),
                },
            )
            .unwrap();

        let total_deposited_by_token_atom: Uint128 = app
            .wrap()
            .query_wasm_smart(
                addr.clone(),
                &QueryMsg::GetTotalDepositedByToken {
                    denom: "atom".to_string(),
                },
            )
            .unwrap();

        assert_eq!(total_deposited_by_token_eth.u128(), 230000000000000000000); // 200 ETH + 30 ETH = 230 ETH
        assert_eq!(total_deposited_by_token_atom.u128(), 700000000000000000000); // 300 ATOM + 400 ATOM = 700 ATOM

        app.set_block(BlockInfo {
            height: 0,
            time: Timestamp::from_seconds(now + YEAR_IN_SECONDS),
            chain_id: "custom_chain_id".to_string(),
        });

        let total_deposited_by_token_eth: Uint128 = app
            .wrap()
            .query_wasm_smart(
                addr.clone(),
                &QueryMsg::GetTotalDepositedByToken {
                    denom: "eth".to_string(),
                },
            )
            .unwrap();

        let total_deposited_by_token_atom: Uint128 = app
            .wrap()
            .query_wasm_smart(
                addr.clone(),
                &QueryMsg::GetTotalDepositedByToken {
                    denom: "atom".to_string(),
                },
            )
            .unwrap();

        // no interest is accrued because no one has yet taken the borrowing
        assert_eq!(total_deposited_by_token_eth.u128(), 230000000000000000000); // 230 ETH
        assert_eq!(total_deposited_by_token_atom.u128(), 700000000000000000000); // 700 ATOM

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

        app.set_block(BlockInfo {
            height: 0,
            time: Timestamp::from_seconds(now + 2*YEAR_IN_SECONDS),
            chain_id: "custom_chain_id".to_string(),
        });

        let total_deposited_by_token_eth: Uint128 = app
            .wrap()
            .query_wasm_smart(
                addr.clone(),
                &QueryMsg::GetTotalDepositedByToken {
                    denom: "eth".to_string(),
                },
            )
            .unwrap();

        let total_deposited_by_token_atom: Uint128 = app
            .wrap()
            .query_wasm_smart(
                addr.clone(),
                &QueryMsg::GetTotalDepositedByToken {
                    denom: "atom".to_string(),
                },
            )
            .unwrap();

        // interest accrued for the year is included in the total balance of deposits
        assert!(total_deposited_by_token_eth.u128() > 230930000000000000000); // > 230.93 ETH (~0.4% liquidity rate)
        assert!(total_deposited_by_token_atom.u128() > 710140000000000000000); // > 710.14 ATOM  (~1.45% liquidity rate)
    }
}
