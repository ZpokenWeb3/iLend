#[cfg(test)]
mod tests {
    use crate::utils::success_deposit_as_collateral_of_diff_token_with_prices;
    use cosmwasm_std::{Addr, BlockInfo, Timestamp, Uint128};
    use cw_multi_test::Executor;
    use master_contract::msg::{ExecuteMsg, QueryMsg, UserBorrowingInfo};
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn test_get_user_borrowing_info() {
        const TOKENS_DECIMALS: u32 = 18;
        const BORROW_AMOUNT_ETH: u128 = 50 * 10u128.pow(TOKENS_DECIMALS); // 50 ETH
        const BORROW_AMOUNT_ATOM: u128 = 200 * 10u128.pow(TOKENS_DECIMALS); // 200 ATOM

        // contract reserves: 1000 ETH and 1000 ATOM
        // user deposited 200 ETH and 300 ATOM
        let (mut app, addr) = success_deposit_as_collateral_of_diff_token_with_prices();

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let user_borrowing_info_eth: UserBorrowingInfo = app
            .wrap()
            .query_wasm_smart(
                addr.clone(),
                &QueryMsg::GetUserBorrowingInfo {
                    address: "user".to_string(),
                    denom: "eth".to_string(),
                },
            )
            .unwrap();

        let user_borrowing_info_atom: UserBorrowingInfo = app
            .wrap()
            .query_wasm_smart(
                addr.clone(),
                &QueryMsg::GetUserBorrowingInfo {
                    address: "user".to_string(),
                    denom: "atom".to_string(),
                },
            )
            .unwrap();

        // user hasn't borrowed anything yet
        assert_eq!(user_borrowing_info_eth.borrowed_amount.u128(), 0);
        // if borrowed_amount == 0 then returns the current interest rate (5%)
        assert_eq!(
            user_borrowing_info_eth.average_interest_rate.u128(),
            5000000000000000000
        );
        assert!(user_borrowing_info_eth.timestamp < Timestamp::from_seconds(now));

        // user hasn't borrowed anything yet
        assert_eq!(user_borrowing_info_atom.borrowed_amount.u128(), 0);
        // if borrowed_amount == 0 then returns the current interest rate (5%)
        assert_eq!(
            user_borrowing_info_atom.average_interest_rate.u128(),
            5000000000000000000
        );
        assert!(user_borrowing_info_atom.timestamp < Timestamp::from_seconds(now));

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

        let user_borrowing_info_eth: UserBorrowingInfo = app
            .wrap()
            .query_wasm_smart(
                addr.clone(),
                &QueryMsg::GetUserBorrowingInfo {
                    address: "user".to_string(),
                    denom: "eth".to_string(),
                },
            )
            .unwrap();

        let user_borrowing_info_atom: UserBorrowingInfo = app
            .wrap()
            .query_wasm_smart(
                addr.clone(),
                &QueryMsg::GetUserBorrowingInfo {
                    address: "user".to_string(),
                    denom: "atom".to_string(),
                },
            )
            .unwrap();

        assert_eq!(
            user_borrowing_info_eth.borrowed_amount.u128(),
            50000000000000000000
        ); // 50 ETH
        assert_eq!(
            user_borrowing_info_eth.average_interest_rate.u128(),
            5000000000000000000
        ); // 5%
        assert_eq!(
            user_borrowing_info_eth.timestamp,
            Timestamp::from_seconds(now)
        );

        assert_eq!(
            user_borrowing_info_atom.borrowed_amount.u128(),
            200000000000000000000
        ); // 200 ATOM
        assert_eq!(
            user_borrowing_info_atom.average_interest_rate.u128(),
            5000000000000000000
        ); // 5%
        assert_eq!(
            user_borrowing_info_atom.timestamp,
            Timestamp::from_seconds(now)
        );
    }
}
