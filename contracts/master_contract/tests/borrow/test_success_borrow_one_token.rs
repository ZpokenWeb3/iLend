#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::success_deposit_of_diff_token_with_prices;
    use cosmwasm_std::{Addr, Uint128};
    use cw_multi_test::Executor;
    use master_contract::msg::{ExecuteMsg, GetBalanceResponse, GetBorrowsResponse, GetSupportedTokensResponse, QueryMsg, RepayInfo};
    use std::fmt::format;

    #[test]
    fn test_success_borrow_one_token() {
        const INIT_BALANCE_FIRST_TOKEN: u128 = 1000;
        const INIT_BALANCE_SECOND_TOKEN: u128 = 1000;

        const DEPOSIT_OF_FIRST_TOKEN: u128 = 200;
        const DEPOSIT_OF_SECOND_TOKEN: u128 = 300;

        const BORROW_SECOND_TOKEN: u128 = 300;

        /*
        price eth 1500
        price atom 10

        deposited eth 200 * 1500 = 300_000 $

        borrowed atom 300 * 10 = 3_000 $
        */

        let (mut app, addr) = success_deposit_of_diff_token_with_prices();

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

        let user_borrowed_balance: GetBorrowsResponse = app
            .wrap()
            .query_wasm_smart(
                addr.clone(),
                &QueryMsg::GetBorrows {
                    address: "user".to_string(),
                    denom: "atom".to_string(),
                },
            )
            .unwrap();

        assert_eq!(user_borrowed_balance.borrows.u128(), BORROW_SECOND_TOKEN);

        assert_eq!(
            app.wrap()
                .query_balance("user", "atom")
                .unwrap()
                .amount
                .u128(),
            INIT_BALANCE_SECOND_TOKEN + BORROW_SECOND_TOKEN
        );

        let repay_info_for_one_token: RepayInfo = app
            .wrap()
            .query_wasm_smart(
                addr.clone(),
                &QueryMsg::GetRepayInfo {
                    address: "user".to_string(),
                    denom: "atom".to_string(),
                },
            )
            .unwrap();

        // mocking interest rate
        assert_eq!(repay_info_for_one_token.borrowed_amount.u128(), BORROW_SECOND_TOKEN);
        assert_eq!(repay_info_for_one_token.accumulated_interest.u128(), BORROW_SECOND_TOKEN / 8);
    }
}
