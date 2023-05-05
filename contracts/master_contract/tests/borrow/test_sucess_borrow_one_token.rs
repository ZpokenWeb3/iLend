#[cfg(test)]
mod tests {
    //     use super::*;
    use crate::utils::success_deposit_of_diff_token_with_prices;
    use cosmwasm_std::{
        Addr,
        BlockInfo,
        Timestamp,
        Uint128,
//         Uint64
    };
    use cw_multi_test::Executor;
    use master_contract::msg::{
        ExecuteMsg,
        GetBalanceResponse,
        GetBorrowAmountWithInterestResponse,
//         GetSupportedTokensResponse,
//         GetTotalBorrowedUsdResponse,
//         GetTotalDepositedUsdResponse,
        QueryMsg,
//         UserBorrowingInfo,
    };
//     use near_sdk::json_types::U128;
//     use std::fmt::format;

    #[test]
    fn test_success_borrow_one_token() {
        const DECIMAL_FRACTIONAL: Uint128 = Uint128::new(1_000_000_000_000_000_000u128); // 1*10**18

        //         const INIT_BALANCE_FIRST_TOKEN: u128 = 1000 * DECIMAL_FRACTIONAL.u128();
        const INIT_BALANCE_SECOND_TOKEN: u128 = 1000 * DECIMAL_FRACTIONAL.u128();

        //         const DEPOSIT_OF_FIRST_TOKEN: u128 = 200 * DECIMAL_FRACTIONAL.u128();
        const DEPOSIT_OF_SECOND_TOKEN: u128 = 300 * DECIMAL_FRACTIONAL.u128();

        const BORROW_SECOND_TOKEN: u128 = 300 * DECIMAL_FRACTIONAL.u128();

        /*
        price eth 1500
        price atom 10

        deposited eth 200 * 1500 = 300_000 $

        borrowed atom 300 * 10 = 3_000 $
        */

        let (mut app, addr) = success_deposit_of_diff_token_with_prices();

        app.set_block(BlockInfo {
            height: 0,
            time: Timestamp::from_seconds(0),
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
            time: Timestamp::from_seconds(31536000),
            chain_id: "custom_chain_id".to_string(),
        });

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

        println!("{:?}", user_borrowed_balance);


        assert_ne!(user_borrowed_balance.amount.u128(), BORROW_SECOND_TOKEN);
        assert_eq!(user_borrowed_balance.amount.u128(), BORROW_SECOND_TOKEN * 105 / 100);
    }
}
