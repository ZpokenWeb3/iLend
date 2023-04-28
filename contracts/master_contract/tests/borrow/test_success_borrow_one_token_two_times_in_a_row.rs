#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::success_deposit_of_diff_token_with_prices;
    use cosmwasm_std::{Addr, Uint128};
    use cw_multi_test::Executor;
    use master_contract::msg::{
        ExecuteMsg, GetBalanceResponse, GetBorrowAmountWithInterestResponse, GetSupportedTokensResponse, QueryMsg,
    };
    use std::fmt::format;

    #[test]
    fn test_success_borrow_one_token() {
        const INIT_BALANCE_FIRST_TOKEN: u128 = 1000;
        const INIT_BALANCE_SECOND_TOKEN: u128 = 1000;

        const DEPOSIT_OF_FIRST_TOKEN: u128 = 200;
        const DEPOSIT_OF_SECOND_TOKEN: u128 = 300;

        const BORROW_SECOND_TOKEN_FIRST_PART: u128 = 300;
        const BORROW_SECOND_TOKEN_SECOND_PART: u128 = 200;

        /*
        price eth 1500
        price atom 10

        deposited eth 200 * 1500 = 300_000 $

        first borrowed atom 300 * 10 = 3_000 $
        second borrowed atom 200 * 10 = 2_000 $
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
                amount: Uint128::from(BORROW_SECOND_TOKEN_FIRST_PART),
            },
            &[],
        )
        .unwrap();

        let user_borrowed_balance: GetBorrowAmountWithInterestResponse = app
            .wrap()
            .query_wasm_smart(
                addr.clone(),
                &QueryMsg::GetBorrowAmountWithInterest {
                    address: "user".to_string(),
                    denom: "atom".to_string(),
                },
            )
            .unwrap();

        assert_eq!(
            user_borrowed_balance.amount.u128(),
            BORROW_SECOND_TOKEN_FIRST_PART
        );

        assert_eq!(
            app.wrap()
                .query_balance("user", "atom")
                .unwrap()
                .amount
                .u128(),
            INIT_BALANCE_SECOND_TOKEN + BORROW_SECOND_TOKEN_FIRST_PART
        );

        app.execute_contract(
            Addr::unchecked("user"),
            addr.clone(),
            &ExecuteMsg::Borrow {
                denom: "atom".to_string(),
                amount: Uint128::from(BORROW_SECOND_TOKEN_SECOND_PART),
            },
            &[],
        )
        .unwrap();

        let user_borrowed_balance: GetBorrowAmountWithInterestResponse = app
            .wrap()
            .query_wasm_smart(
                addr.clone(),
                &QueryMsg::GetBorrowAmountWithInterest {
                    address: "user".to_string(),
                    denom: "atom".to_string(),
                },
            )
            .unwrap();

        assert_eq!(
            user_borrowed_balance.amount.u128(),
            BORROW_SECOND_TOKEN_FIRST_PART + BORROW_SECOND_TOKEN_SECOND_PART
        );

        assert_eq!(
            app.wrap()
                .query_balance("user", "atom")
                .unwrap()
                .amount
                .u128(),
            INIT_BALANCE_SECOND_TOKEN
                + BORROW_SECOND_TOKEN_SECOND_PART
                + BORROW_SECOND_TOKEN_FIRST_PART
        );
    }
}
