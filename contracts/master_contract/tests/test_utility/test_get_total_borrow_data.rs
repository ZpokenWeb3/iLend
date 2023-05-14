#[cfg(test)]
mod tests {
    use crate::utils::success_deposit_as_collateral_of_diff_token_with_prices;
    use cosmwasm_std::{Addr, BlockInfo, Timestamp, Uint128};
    use cw_multi_test::Executor;
    use master_contract::msg::{ExecuteMsg, QueryMsg, TotalBorrowData};
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn test_get_total_borrow_data() {
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

        let total_borrow_data_eth: TotalBorrowData = app
            .wrap()
            .query_wasm_smart(
                addr.clone(),
                &QueryMsg::GetTotalBorrowData {
                    denom: "eth".to_string(),
                },
            )
            .unwrap();

        let total_borrow_data_atom: TotalBorrowData = app
            .wrap()
            .query_wasm_smart(
                addr.clone(),
                &QueryMsg::GetTotalBorrowData {
                    denom: "atom".to_string(),
                },
            )
            .unwrap();

        // user hasn't borrowed anything yet
        assert_eq!(total_borrow_data_eth.denom, "eth");
        assert_eq!(total_borrow_data_eth.total_borrowed_amount, 0);
        assert_eq!(total_borrow_data_eth.expected_annual_interest_income, 0);
        assert_eq!(total_borrow_data_eth.average_interest_rate, 0);
        assert!(total_borrow_data_eth.timestamp < Timestamp::from_seconds(now));

        // user hasn't borrowed anything yet
        assert_eq!(total_borrow_data_atom.denom, "atom");
        assert_eq!(total_borrow_data_atom.total_borrowed_amount, 0);
        assert_eq!(total_borrow_data_atom.expected_annual_interest_income, 0);
        assert_eq!(total_borrow_data_atom.average_interest_rate, 0);
        assert!(total_borrow_data_atom.timestamp < Timestamp::from_seconds(now));

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

        let total_borrow_data_eth: TotalBorrowData = app
            .wrap()
            .query_wasm_smart(
                addr.clone(),
                &QueryMsg::GetTotalBorrowData {
                    denom: "eth".to_string(),
                },
            )
            .unwrap();

        let total_borrow_data_atom: TotalBorrowData = app
            .wrap()
            .query_wasm_smart(
                addr.clone(),
                &QueryMsg::GetTotalBorrowData {
                    denom: "atom".to_string(),
                },
            )
            .unwrap();

        assert_eq!(total_borrow_data_eth.denom, "eth");
        assert_eq!(
            total_borrow_data_eth.total_borrowed_amount,
            50000000000000000000
        ); // 50 ETH
        assert_eq!(
            total_borrow_data_eth.expected_annual_interest_income,
            2500000000000000000
        ); // 2.5 ETH (5% borrow APY)
        assert_eq!(
            total_borrow_data_eth.average_interest_rate,
            5000000000000000000
        ); // 5%
        assert_eq!(
            total_borrow_data_eth.timestamp,
            Timestamp::from_seconds(now)
        );

        assert_eq!(total_borrow_data_atom.denom, "atom");
        assert_eq!(
            total_borrow_data_atom.total_borrowed_amount,
            200000000000000000000
        ); // 200 ETH
        assert_eq!(
            total_borrow_data_atom.expected_annual_interest_income,
            10000000000000000000
        ); // 10 ETH (5% borrow APY)
        assert_eq!(
            total_borrow_data_atom.average_interest_rate,
            5000000000000000000
        ); // 5%
        assert_eq!(
            total_borrow_data_atom.timestamp,
            Timestamp::from_seconds(now)
        );
    }
}
