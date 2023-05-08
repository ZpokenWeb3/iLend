#[cfg(test)]
mod tests {
    //     use super::*;
    use crate::utils::success_borrow_setup;
    use cosmwasm_std::{coins, Addr, BlockInfo, Timestamp};
    use cw_multi_test::Executor;
    use master_contract::msg::{ExecuteMsg, GetBorrowAmountWithInterestResponse, QueryMsg};

    #[test]
    fn test_success_repay() {
        let (mut app, addr) = success_borrow_setup();

        app.set_block(BlockInfo {
            height: 542,
            time: Timestamp::from_seconds(31536000 + 10000),
            chain_id: "custom_chain_id".to_string(),
        });

        let get_amount_with_interest_data: GetBorrowAmountWithInterestResponse = app
            .wrap()
            .query_wasm_smart(
                addr.clone(),
                &QueryMsg::GetBorrowAmountWithInterest {
                    address: "user".to_string(),
                    denom: "eth".to_string(),
                },
            )
            .unwrap();

        let amount_to_repay_with_interest = get_amount_with_interest_data.amount.u128();

        let underlying_balance_before_repay = app
            .wrap()
            .query_balance(&addr, "eth")
            .unwrap()
            .amount
            .u128();

        app.execute_contract(
            Addr::unchecked("user"),
            addr.clone(),
            &ExecuteMsg::Repay {},
            &coins(amount_to_repay_with_interest * 2, "eth"),
        )
        .unwrap();

        let underlying_balance_after_repay = app
            .wrap()
            .query_balance(&addr, "eth")
            .unwrap()
            .amount
            .u128();

        // paying only what we supposed to, not twice as much
        assert_eq!(
            underlying_balance_after_repay - amount_to_repay_with_interest,
            underlying_balance_before_repay
        );

        let get_borrow_amount_with_interest_response: GetBorrowAmountWithInterestResponse = app
            .wrap()
            .query_wasm_smart(
                addr.clone(),
                &QueryMsg::GetBorrowAmountWithInterest {
                    address: "user".to_string(),
                    denom: "eth".to_string(),
                },
            )
            .unwrap();

        assert_eq!(get_borrow_amount_with_interest_response.amount.u128(), 0);
    }
}
