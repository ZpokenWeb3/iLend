#[cfg(test)]
mod tests {
    use cosmwasm_std::Addr;
    use cw_multi_test::Executor;

    use crate::utils::success_deposit_of_diff_token_with_prices;
    use cosmwasm_std::Uint128;
    use master_contract::msg::{ExecuteMsg, GetBalanceResponse, QueryMsg};

    #[test]
    fn test_success_redeem_diff_token() {
        const DECIMAL_FRACTIONAL: Uint128 = Uint128::new(1_000_000_000_000_000_000u128); // 1*10**18

        const INIT_BALANCE_FIRST_TOKEN: u128 = 1000 * DECIMAL_FRACTIONAL.u128();
        const INIT_BALANCE_SECOND_TOKEN: u128 = 1_000_000 * DECIMAL_FRACTIONAL.u128(); // 1M ATOM

        const DEPOSIT_OF_FIRST_TOKEN: u128 = 200 * DECIMAL_FRACTIONAL.u128();
        const DEPOSIT_OF_SECOND_TOKEN: u128 = 300 * DECIMAL_FRACTIONAL.u128();

        const WITHDRAW_AMOUNT_FIRST_TOKEN: u128 = 100 * DECIMAL_FRACTIONAL.u128();
        const WITHDRAW_AMOUNT_SECOND_TOKEN: u128 = 150 * DECIMAL_FRACTIONAL.u128();

        let (mut app, addr) = success_deposit_of_diff_token_with_prices();

        app.execute_contract(
            Addr::unchecked("user"),
            addr.clone(),
            &ExecuteMsg::Redeem {
                denom: "eth".to_string(),
                amount: Uint128::from(WITHDRAW_AMOUNT_FIRST_TOKEN),
            },
            &[],
        )
        .unwrap();

        app.execute_contract(
            Addr::unchecked("user"),
            addr.clone(),
            &ExecuteMsg::Redeem {
                denom: "atom".to_string(),
                amount: Uint128::from(WITHDRAW_AMOUNT_SECOND_TOKEN),
            },
            &[],
        )
        .unwrap();

        let user_deposited_balance_of_first_token: GetBalanceResponse = app
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
            user_deposited_balance_of_first_token.balance.u128(),
            DEPOSIT_OF_FIRST_TOKEN - WITHDRAW_AMOUNT_FIRST_TOKEN
        );

        assert_eq!(
            app.wrap()
                .query_balance("user", "eth")
                .unwrap()
                .amount
                .u128(),
            INIT_BALANCE_FIRST_TOKEN - DEPOSIT_OF_FIRST_TOKEN + WITHDRAW_AMOUNT_FIRST_TOKEN
        );

        let user_deposited_balance_of_second_token: GetBalanceResponse = app
            .wrap()
            .query_wasm_smart(
                addr.clone(),
                &QueryMsg::GetDeposit {
                    address: "user".to_string(),
                    denom: "atom".to_string(),
                },
            )
            .unwrap();

        assert_eq!(
            user_deposited_balance_of_second_token.balance.u128(),
            DEPOSIT_OF_SECOND_TOKEN - WITHDRAW_AMOUNT_SECOND_TOKEN
        );

        assert_eq!(
            app.wrap()
                .query_balance("user", "atom")
                .unwrap()
                .amount
                .u128(),
            INIT_BALANCE_SECOND_TOKEN - DEPOSIT_OF_SECOND_TOKEN + WITHDRAW_AMOUNT_SECOND_TOKEN
        );
    }
}
