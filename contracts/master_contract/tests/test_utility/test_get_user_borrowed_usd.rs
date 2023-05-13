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
        QueryMsg,
    };
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn test_get_user_borrowed_usd() {
        const TOKENS_DECIMALS: u32 = 18;
        const BORROW_AMOUNT_ETH: u128 = 50 * 10u128.pow(TOKENS_DECIMALS); // 50 ETH
        const BORROW_AMOUNT_ATOM: u128 = 200 * 10u128.pow(TOKENS_DECIMALS); // 200 ATOM

        const YEAR_IN_SECONDS: u64 = 31536000;

        // contract reserves: 1000 ETH and 1000 ATOM
        // user deposited 200 ETH and 300 ATOM
        let (mut app, addr) = success_deposit_as_collateral_of_diff_token_with_prices();

        let user_borrowed_usd: Uint128 = app
            .wrap()
            .query_wasm_smart(
                addr.clone(),
                &QueryMsg::GetUserBorrowedUsd {
                    address: "user".to_string(),
                },
            )
            .unwrap();

        // user hasn't borrowed anything yet
        assert_eq!(user_borrowed_usd.u128(), 0);
        
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

        let user_borrowed_usd: Uint128 = app
            .wrap()
            .query_wasm_smart(
                addr.clone(),
                &QueryMsg::GetUserBorrowedUsd {
                    address: "user".to_string(),
                },
            )
            .unwrap();

        assert_eq!(user_borrowed_usd.u128(), 10200000000000); // 50 ETH * 2000 + 200 ATOM * 10 = 100_000$ + 2_000$ = 102_000$

        app.set_block(BlockInfo {
            height: 0,
            time: Timestamp::from_seconds(now + YEAR_IN_SECONDS),
            chain_id: "custom_chain_id".to_string(),
        });

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

        assert_eq!(user_borrowed_usd.u128(), 10910000000000); // 109_100$ (~7% APY)
    }
}
