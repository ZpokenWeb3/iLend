#[cfg(test)]
mod tests {
    use crate::utils::success_deposit_as_collateral_of_diff_token_with_prices;
    use cosmwasm_std::{Addr, BlockInfo, Timestamp, Uint128};
    use cw_multi_test::Executor;
    use master_contract::msg::{ExecuteMsg, QueryMsg};
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn test_get_user_borrow_amount_with_interest() {
        const TOKENS_DECIMALS: u32 = 18;
        const BORROW_AMOUNT_ETH: u128 = 50 * 10u128.pow(TOKENS_DECIMALS); // 50 ETH
        const BORROW_AMOUNT_ATOM: u128 = 200 * 10u128.pow(TOKENS_DECIMALS); // 200 ATOM

        const YEAR_IN_SECONDS: u64 = 31536000;

        // contract reserves: 1000 ETH and 1000 ATOM
        // user deposited 200 ETH and 300 ATOM
        let (mut app, addr) = success_deposit_as_collateral_of_diff_token_with_prices();

        let user_borrow_amount_with_interest_eth: Uint128 = app
            .wrap()
            .query_wasm_smart(
                addr.clone(),
                &QueryMsg::GetUserBorrowAmountWithInterest {
                    address: "user".to_string(),
                    denom: "eth".to_string(),
                },
            )
            .unwrap();

        let user_borrow_amount_with_interest_atom: Uint128 = app
            .wrap()
            .query_wasm_smart(
                addr.clone(),
                &QueryMsg::GetUserBorrowAmountWithInterest {
                    address: "user".to_string(),
                    denom: "atom".to_string(),
                },
            )
            .unwrap();

        // user hasn't borrowed anything yet
        assert_eq!(user_borrow_amount_with_interest_eth.u128(), 0);
        assert_eq!(user_borrow_amount_with_interest_atom.u128(), 0);

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

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

        let user_borrow_amount_with_interest_eth: Uint128 = app
            .wrap()
            .query_wasm_smart(
                addr.clone(),
                &QueryMsg::GetUserBorrowAmountWithInterest {
                    address: "user".to_string(),
                    denom: "eth".to_string(),
                },
            )
            .unwrap();

        let user_borrow_amount_with_interest_atom: Uint128 = app
            .wrap()
            .query_wasm_smart(
                addr.clone(),
                &QueryMsg::GetUserBorrowAmountWithInterest {
                    address: "user".to_string(),
                    denom: "atom".to_string(),
                },
            )
            .unwrap();

        assert_eq!(
            user_borrow_amount_with_interest_eth.u128(),
            50000000000000000000
        ); // 50 ETH
        assert_eq!(
            user_borrow_amount_with_interest_atom.u128(),
            200000000000000000000
        ); // 200 ATOM

        app.set_block(BlockInfo {
            height: 0,
            time: Timestamp::from_seconds(now + YEAR_IN_SECONDS),
            chain_id: "custom_chain_id".to_string(),
        });

        let user_borrow_amount_with_interest_eth: Uint128 = app
            .wrap()
            .query_wasm_smart(
                addr.clone(),
                &QueryMsg::GetUserBorrowAmountWithInterest {
                    address: "user".to_string(),
                    denom: "eth".to_string(),
                },
            )
            .unwrap();

        let user_borrow_amount_with_interest_atom: Uint128 = app
            .wrap()
            .query_wasm_smart(
                addr.clone(),
                &QueryMsg::GetUserBorrowAmountWithInterest {
                    address: "user".to_string(),
                    denom: "atom".to_string(),
                },
            )
            .unwrap();

        // 50 ETH + 5% borrow APY = 50 ETH + 2.5 ETH = 52.5 ETH
        assert_eq!(
            user_borrow_amount_with_interest_eth.u128(),
            52500000000000000000
        );
        // 200 ATOM + 5% borrow APY = 200 ATOM + 10 ATOM = 210 ATOM
        assert_eq!(
            user_borrow_amount_with_interest_atom.u128(),
            210000000000000000000
        );

        let users_with_borrow: Vec<String> = app
            .wrap()
            .query_wasm_smart(addr.clone(), &QueryMsg::GetAllUsersWithBorrows {})
            .unwrap();

        assert!(!users_with_borrow.is_empty());
    }
}
