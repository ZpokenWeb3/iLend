#[cfg(test)]
mod tests {
    use cosmwasm_std::Addr;
    use cw_multi_test::Executor;

    use cosmwasm_std::Uint128;
    use lending::msg::{ExecuteMsg, GetBalanceResponse, QueryMsg};

    use crate::utils::success_deposit_of_one_token_setup;

    #[test]
    #[should_panic]
    fn test_fail_redeem_more_than_deposited() {
        const DECIMAL_FRACTIONAL: Uint128 = Uint128::new(1_000_000_000_000_000_000u128); // 1*10**18

        const INIT_USER_BALANCE: u128 = 1000 * DECIMAL_FRACTIONAL.u128();
        //         const CONTRACT_RESERVES: u128 = 1000000 * DECIMAL_FRACTIONAL.u128();
        const TOTAL_DEPOSITED: u128 = 500 * DECIMAL_FRACTIONAL.u128();
        const WITHDRAW_AMOUNT: u128 = 700 * DECIMAL_FRACTIONAL.u128();

        // having 500 deposited we want to redeem WITHDRAW_AMOUNT
        let (mut app, lending_contract_addr, _collateral_contract_addr) =
            success_deposit_of_one_token_setup();

        app.execute_contract(
            Addr::unchecked("user"),
            lending_contract_addr.clone(),
            &ExecuteMsg::Redeem {
                denom: "eth".to_string(),
                amount: Uint128::from(WITHDRAW_AMOUNT),
            },
            &[],
        )
        .unwrap();

        let user_deposited_balance: GetBalanceResponse = app
            .wrap()
            .query_wasm_smart(
                lending_contract_addr.clone(),
                &QueryMsg::GetDeposit {
                    address: "user".to_string(),
                    denom: "eth".to_string(),
                },
            )
            .unwrap();

        assert_eq!(user_deposited_balance.balance.u128(), TOTAL_DEPOSITED);

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
