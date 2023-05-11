#[cfg(test)]
mod tests {
    use crate::utils::success_deposit_as_collateral_of_diff_token_with_prices;
    use cosmwasm_std::{
        Addr,
        BlockInfo,
        Timestamp,
        Uint128,
    };
    use cw_multi_test::Executor;
    use master_contract::msg::{
        ExecuteMsg,
        GetBalanceResponse,
        QueryMsg,
    };
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn test_success_borrow_one_token() {
        const DECIMAL_FRACTIONAL: Uint128 = Uint128::new(1_000_000_000_000_000_000u128); // 1*10**18

        const INIT_BALANCE_SECOND_TOKEN: u128 = 1_000_000 * DECIMAL_FRACTIONAL.u128(); // 1M ATOM

        const DEPOSIT_OF_SECOND_TOKEN: u128 = 300 * DECIMAL_FRACTIONAL.u128();

        const BORROW_SECOND_TOKEN: u128 = 300 * DECIMAL_FRACTIONAL.u128();

        /*
        price eth 1500
        price atom 10

        deposited eth 200 * 1500 = 300_000 $

        borrowed atom 300 * 10 = 3_000 $
        */

        // contract reserves: 1000 ETH and 1000 ATOM
        // user deposited 200 ETH and 300 ATOM
        let (mut app, addr) = success_deposit_as_collateral_of_diff_token_with_prices();

        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();

        app.set_block(BlockInfo {
            height: 0,
            time: Timestamp::from_seconds(now),
            chain_id: "custom_chain_id".to_string(),
        });

        app.execute_contract(
            Addr::unchecked("user"),
            addr.clone(),
            &ExecuteMsg::Redeem {
                denom: "atom".to_string(),
                amount: Uint128::from(DEPOSIT_OF_SECOND_TOKEN),
            },
            &[],
        )
        .unwrap();

        let user_deposited_balance_after_redeeming: GetBalanceResponse = app
            .wrap()
            .query_wasm_smart(
                addr.clone(),
                &QueryMsg::GetDeposit {
                    address: "user".to_string(),
                    denom: "atom".to_string(),
                },
            )
            .unwrap();

        assert_eq!(user_deposited_balance_after_redeeming.balance.u128(), 0);

        assert_eq!(
            app.wrap()
                .query_balance("user", "atom")
                .unwrap()
                .amount
                .u128(),
            INIT_BALANCE_SECOND_TOKEN
        );

        app.execute_contract(
            Addr::unchecked("user"),
            addr.clone(),
            &ExecuteMsg::Borrow {
                denom: "atom".to_string(),
                amount: Uint128::from(BORROW_SECOND_TOKEN),
            },
            &[],
        )
        .unwrap();

        app.set_block(BlockInfo {
            height: 2,
            time: Timestamp::from_seconds(now + 31536000),
            chain_id: "custom_chain_id".to_string(),
        });

        let user_borrowed_balance: Uint128 = app
            .wrap()
            .query_wasm_smart(
                addr.clone(),
                &QueryMsg::GetUserBorrowAmountWithInterest {
                    address: "user".to_string(),
                    denom: "atom".to_string(),
                },
            )
            .unwrap();

        assert_ne!(user_borrowed_balance.u128(), BORROW_SECOND_TOKEN);
        assert_eq!(
            user_borrowed_balance.u128(),
            BORROW_SECOND_TOKEN * 105 / 100
        );
    }
}
