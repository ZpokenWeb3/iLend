#[cfg(test)]
mod tests {
    use cosmwasm_std::Addr;
    use cw_multi_test::Executor;

    use cosmwasm_std::Uint128;
    use master_contract::msg::{ExecuteMsg, GetBalanceResponse, QueryMsg};

    use crate::utils::success_deposit_of_one_token_setup;

    #[test]
    fn test_success_redeem_one_token_whole_deposit() {
        const DECIMAL_FRACTIONAL: Uint128 = Uint128::new(1_000_000_000_000_000_000u128); // 1*10**18

        const INIT_USER_BALANCE: u128 = 1000 * DECIMAL_FRACTIONAL.u128();
        //         const CONTRACT_RESERVES: u128 = 1000000 * DECIMAL_FRACTIONAL.u128();
        const FIRST_DEPOSIT_AMOUNT: u128 = 200 * DECIMAL_FRACTIONAL.u128();
        const SECOND_DEPOSIT_AMOUNT: u128 = 300 * DECIMAL_FRACTIONAL.u128();

        // having 500 deposited we want to redeem SECOND_DEPOSIT_AMOUNT
        // so that FIRST_DEPOSIT_AMOUNT is remaining
        let (mut app, addr) = success_deposit_of_one_token_setup();

        app.execute_contract(
            Addr::unchecked("user"),
            addr.clone(),
            &ExecuteMsg::Redeem {
                denom: "eth".to_string(),
                amount: Uint128::from(FIRST_DEPOSIT_AMOUNT + SECOND_DEPOSIT_AMOUNT),
            },
            &[],
        )
        .unwrap();

        let user_deposited_balance_after_first_withdrawal: GetBalanceResponse = app
            .wrap()
            .query_wasm_smart(
                addr.clone(),
                &QueryMsg::GetDeposit {
                    address: "user".to_string(),
                    denom: "eth".to_string(),
                },
            )
            .unwrap();

        assert_eq!(
            user_deposited_balance_after_first_withdrawal.balance.u128(),
            0
        );

        assert_eq!(
            app.wrap()
                .query_balance("user", "eth")
                .unwrap()
                .amount
                .u128(),
            INIT_USER_BALANCE
        )
    }
}
