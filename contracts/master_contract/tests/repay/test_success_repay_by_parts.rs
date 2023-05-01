#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::{success_borrow_setup, success_deposit_of_diff_token_with_prices};
    use cosmwasm_std::{coins, Addr, Uint128};
    use cw_multi_test::Executor;
    use master_contract::msg::{
        ExecuteMsg, GetBorrowAmountWithInterestResponse, GetTotalBorrowedUsdResponse,
        GetUserBorrowedUsdResponse, GetUserDepositedUsdResponse, QueryMsg, UserBorrowingInfo,
    };

    #[test]
    fn test_success_repay() {
        const TOKEN_DECIMAL: u128 = 10u128.pow(18);
        const BORROW_OF_FIRST_TOKEN: u128 = 50 * TOKEN_DECIMAL;

        let (mut app, addr) = success_borrow_setup();

        let repay_info: UserBorrowingInfo = app
            .wrap()
            .query_wasm_smart(
                addr.clone(),
                &QueryMsg::GetUserBorrowingInfo {
                    address: "user".to_string(),
                    denom: "eth".to_string(),
                },
            )
            .unwrap();

        assert_eq!(
            repay_info.accumulated_interest.u128(),
            BORROW_OF_FIRST_TOKEN / 8
        );
        assert_eq!(repay_info.borrowed_amount.u128(), BORROW_OF_FIRST_TOKEN);

        app.execute_contract(
            Addr::unchecked("user"),
            addr.clone(),
            &ExecuteMsg::Repay {},
            &coins(
                repay_info.accumulated_interest.u128() + BORROW_OF_FIRST_TOKEN / 2,
                "eth",
            ),
        )
        .unwrap();

        let repay_info_after_repay: UserBorrowingInfo = app
            .wrap()
            .query_wasm_smart(
                addr.clone(),
                &QueryMsg::GetUserBorrowingInfo {
                    address: "user".to_string(),
                    denom: "eth".to_string(),
                },
            )
            .unwrap();

        assert_eq!(repay_info_after_repay.accumulated_interest.u128(), 0);
        assert_eq!(
            repay_info_after_repay.borrowed_amount.u128(),
            BORROW_OF_FIRST_TOKEN / 2
        );

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

        assert_eq!(
            user_borrowed_balance.amount.u128(),
            BORROW_OF_FIRST_TOKEN / 2
        );

        app.execute_contract(
            Addr::unchecked("user"),
            addr.clone(),
            &ExecuteMsg::Repay {},
            &coins(BORROW_OF_FIRST_TOKEN / 2, "eth"),
        )
        .unwrap();

        let repay_info_after_repay: UserBorrowingInfo = app
            .wrap()
            .query_wasm_smart(
                addr.clone(),
                &QueryMsg::GetUserBorrowingInfo {
                    address: "user".to_string(),
                    denom: "eth".to_string(),
                },
            )
            .unwrap();

        assert_eq!(repay_info_after_repay.accumulated_interest.u128(), 0);
        assert_eq!(repay_info_after_repay.borrowed_amount.u128(), 0);

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
