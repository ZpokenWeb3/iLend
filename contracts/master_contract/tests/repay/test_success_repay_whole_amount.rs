#[cfg(test)]
mod tests {
    //     use super::*;
    use crate::utils::{
        success_borrow_setup,
//         success_deposit_of_diff_token_with_prices
    };
    use cosmwasm_std::{coins, Addr, BlockInfo, Timestamp};
    use cw_multi_test::Executor;
    use master_contract::msg::{
        ExecuteMsg, GetBorrowAmountWithInterestResponse, QueryMsg, UserBorrowingInfo,
    };

    #[test]
    fn test_success_repay() {
        const TOKEN_DECIMAL: u128 = 10u128.pow(18);
        const BORROW_OF_FIRST_TOKEN: u128 = 50 * TOKEN_DECIMAL;

        let (mut app, addr) = success_borrow_setup();

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

        assert_eq!(get_amount_with_interest_data.amount.u128(), BORROW_OF_FIRST_TOKEN * 105 / 100);

        app.execute_contract(
            Addr::unchecked("user"),
            addr.clone(),
            &ExecuteMsg::Repay {},
            &coins(
                get_amount_with_interest_data.amount.u128() + BORROW_OF_FIRST_TOKEN,
                "eth",
            ),
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
        assert_eq!(get_borrow_amount_with_interest_response.amount.u128(), 0);

        let user_borrowed_balance: GetBorrowAmountWithInterestResponse = app
            .wrap()
            .query_wasm_smart(
                addr.clone(),
                &QueryMsg::GetBorrowAmountWithInterest {
                    address: "user".to_string(),
                    denom: "eth".to_string(),
                },
            )
            .unwrap();

        assert_eq!(user_borrowed_balance.amount.u128(), 0);
    }
}
