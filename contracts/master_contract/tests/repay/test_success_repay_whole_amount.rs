#[cfg(test)]
mod tests {
    //     use super::*;
    use crate::utils::success_borrow_setup;
    use cosmwasm_std::{coins, Addr, BlockInfo, Timestamp};
    use cw_multi_test::Executor;
    use master_contract::msg::{ExecuteMsg, GetBorrowAmountWithInterestResponse, QueryMsg};

    #[test]
    fn test_success_repay() {
        const TOKEN_DECIMAL: u128 = 10u128.pow(18);
        const BORROW_OF_FIRST_TOKEN: u128 = 50 * TOKEN_DECIMAL;

        let (mut app, addr) = success_borrow_setup();

        app.set_block(BlockInfo {
            height: 542,
            time: Timestamp::from_seconds(3153600),
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

        app.execute_contract(
            Addr::unchecked("user"),
            addr.clone(),
            &ExecuteMsg::Repay {},
            &coins(amount_to_repay_with_interest, "eth"),
        )
        .unwrap();

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
