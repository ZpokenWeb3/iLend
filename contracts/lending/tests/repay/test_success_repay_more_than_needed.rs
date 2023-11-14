#[cfg(test)]
mod tests {
    use crate::utils::success_borrow_setup;
    use cosmwasm_std::{coins, Addr, BlockInfo, Timestamp, Uint128};
    use cw_multi_test::Executor;
    use lending::msg::{ExecuteMsg, QueryMsg};
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn test_success_repay_more_than_needed() {
        // user borrowed 50 ETH
        let (mut app, lending_contract_addr, _collateral_contract_addr) = success_borrow_setup();

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        app.set_block(BlockInfo {
            height: 542,
            time: Timestamp::from_seconds(now + 31536000 + 10000),
            chain_id: "custom_chain_id".to_string(),
        });

        let user_borrow_amount_with_interest: Uint128 = app
            .wrap()
            .query_wasm_smart(
                lending_contract_addr.clone(),
                &QueryMsg::GetUserBorrowAmountWithInterest {
                    address: "user".to_string(),
                    denom: "eth".to_string(),
                },
            )
            .unwrap();

        let amount_to_repay_with_interest = user_borrow_amount_with_interest.u128();

        app.execute_contract(
            Addr::unchecked("user"),
            lending_contract_addr.clone(),
            &ExecuteMsg::Repay {},
            &coins(amount_to_repay_with_interest * 2, "eth"),
        )
        .unwrap();

        let user_borrow_amount_with_interest: Uint128 = app
            .wrap()
            .query_wasm_smart(
                lending_contract_addr.clone(),
                &QueryMsg::GetUserBorrowAmountWithInterest {
                    address: "user".to_string(),
                    denom: "eth".to_string(),
                },
            )
            .unwrap();

        assert_eq!(user_borrow_amount_with_interest.u128(), 0);
    }
}
