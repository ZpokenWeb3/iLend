#[cfg(test)]
mod tests {
    //     use super::*;
    use crate::utils::success_borrow_setup;
    use cosmwasm_std::{coins, Addr, BlockInfo, Timestamp};
    use cw_multi_test::Executor;
    use master_contract::msg::{
        ExecuteMsg,
        GetBorrowAmountWithInterestResponse,
        //         GetTotalBorrowedUsdResponse,
        //         GetUserBorrowedUsdResponse,
        //         GetUserDepositedUsdResponse,
        QueryMsg,
        UserBorrowingInfo,
    };

    #[test]
    fn test_success_repay_by_parts() {
        const TOKEN_DECIMAL: u128 = 10u128.pow(18);
        const BORROW_OF_FIRST_TOKEN: u128 = 50 * TOKEN_DECIMAL;

        let (mut app, addr) = success_borrow_setup();

        app.set_block(BlockInfo {
            height: 542,
            time: Timestamp::from_seconds(3153600 + 5000),
            chain_id: "custom_chain_id".to_string(),
        });

        let borrow_info_before_first_repay: GetBorrowAmountWithInterestResponse = app
            .wrap()
            .query_wasm_smart(
                addr.clone(),
                &QueryMsg::GetBorrowAmountWithInterest {
                    address: "user".to_string(),
                    denom: "eth".to_string(),
                },
            )
            .unwrap();

        println!("{}", borrow_info_before_first_repay.amount.u128());

        app.execute_contract(
            Addr::unchecked("user"),
            addr.clone(),
            &ExecuteMsg::Repay {},
            &coins(borrow_info_before_first_repay.amount.u128() / 2, "eth"),
        )
        .unwrap();

        let borrow_info_after_first_repay: GetBorrowAmountWithInterestResponse = app
            .wrap()
            .query_wasm_smart(
                addr.clone(),
                &QueryMsg::GetBorrowAmountWithInterest {
                    address: "user".to_string(),
                    denom: "eth".to_string(),
                },
            )
            .unwrap();

        println!("{}", borrow_info_after_first_repay.amount.u128());

        app.execute_contract(
            Addr::unchecked("user"),
            addr.clone(),
            &ExecuteMsg::Repay {},
            &coins(borrow_info_before_first_repay.amount.u128() / 2, "eth"),
        )
        .unwrap();

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
        // 52065639950985158186
        assert_eq!(user_borrowed_balance.amount.u128(), 0);
    }
}
