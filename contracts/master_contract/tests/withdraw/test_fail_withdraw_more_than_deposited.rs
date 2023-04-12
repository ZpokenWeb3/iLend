#[cfg(test)]
mod tests {
    use cosmwasm_std::Addr;
    use cw_multi_test::Executor;

    use cosmwasm_std::Uint128;
    use master_contract::msg::{ExecuteMsg, QueryMsg};

    use crate::utils::success_deposit_of_one_token_setup;

    #[test]
    #[should_panic]
    fn test_fail_more_than_deposited() {
        const INIT_USER_BALANCE: u128 = 1000;
        const CONTRACT_RESERVES: u128 = 1000000;
        const TOTAL_DEPOSITED: u128 = 500;
        const WITHDRAW_AMOUNT: u128 = 700;

        // having 500 deposited we want to withdraw WITHDRAW_AMOUNT
        let (mut app, addr) = success_deposit_of_one_token_setup();

        app.execute_contract(
            Addr::unchecked("user"),
            addr.clone(),
            &ExecuteMsg::Withdraw {
                denom: "eth".to_string(),
                amount: Uint128::from(WITHDRAW_AMOUNT),
            },
            &[],
        )
        .unwrap();

        let user_deposited_balance: Uint128 = app
            .wrap()
            .query_wasm_smart(
                addr.clone(),
                &QueryMsg::GetDeposit {
                    address: "user".to_string(),
                    denom: "eth".to_string(),
                },
            )
            .unwrap();

        assert_eq!(user_deposited_balance.u128(), TOTAL_DEPOSITED);

        assert_eq!(
            app.wrap()
                .query_balance("user", "eth")
                .unwrap()
                .amount
                .u128(),
            INIT_USER_BALANCE - TOTAL_DEPOSITED
        );
    }
}
