#[cfg(test)]
mod tests {

    use crate::utils::success_setup_collateral_vault_and_margin_contract;
    use cosmwasm_std::{coins, Addr, Uint128};
    use cw_multi_test::Executor;
    use margin_positions::msg::ExecuteMsg;
    use margin_positions::msg::QueryMsg;

    #[test]
    fn test_success_deposit_one_token_in_a_row() {
        const TOKENS_DECIMALS: u32 = 18;
        const INIT_USER_BALANCE: u128 = 1000 * 10u128.pow(TOKENS_DECIMALS);
        const RESERVE_AMOUNT: u128 = 1000 * 10u128.pow(TOKENS_DECIMALS);
        const FIRST_DEPOSIT_AMOUNT_ETH: u128 = 200 * 10u128.pow(TOKENS_DECIMALS);
        const SECOND_DEPOSIT_AMOUNT_ETH: u128 = 300 * 10u128.pow(TOKENS_DECIMALS);

        let (mut app, margin_positions_addr, collateral_contract_addr) =
            success_setup_collateral_vault_and_margin_contract();

        app.execute_contract(
            Addr::unchecked("user"),
            margin_positions_addr.clone(),
            &ExecuteMsg::Deposit {},
            &coins(FIRST_DEPOSIT_AMOUNT_ETH, "eth"),
        )
        .unwrap();

        let user_deposited_balance: Uint128 = app
            .wrap()
            .query_wasm_smart(
                margin_positions_addr.clone(),
                &QueryMsg::GetDeposit {
                    user: "user".to_string(),
                    denom: "eth".to_string(),
                },
            )
            .unwrap();

        assert_eq!(user_deposited_balance.u128(), FIRST_DEPOSIT_AMOUNT_ETH);

        assert_eq!(
            app.wrap()
                .query_balance("user", "eth")
                .unwrap()
                .amount
                .u128(),
            INIT_USER_BALANCE - FIRST_DEPOSIT_AMOUNT_ETH
        );

        assert_eq!(
            app.wrap()
                .query_balance(collateral_contract_addr.clone(), "eth")
                .unwrap()
                .amount
                .u128(),
            RESERVE_AMOUNT + FIRST_DEPOSIT_AMOUNT_ETH
        );

        app.execute_contract(
            Addr::unchecked("user"),
            margin_positions_addr.clone(),
            &ExecuteMsg::Deposit {},
            &coins(SECOND_DEPOSIT_AMOUNT_ETH, "eth"),
        )
        .unwrap();

        let user_deposited_balance: Uint128 = app
            .wrap()
            .query_wasm_smart(
                margin_positions_addr.clone(),
                &QueryMsg::GetDeposit {
                    user: "user".to_string(),
                    denom: "eth".to_string(),
                },
            )
            .unwrap();

        assert_eq!(
            user_deposited_balance.u128(),
            FIRST_DEPOSIT_AMOUNT_ETH + SECOND_DEPOSIT_AMOUNT_ETH
        );

        assert_eq!(
            app.wrap()
                .query_balance("user", "eth")
                .unwrap()
                .amount
                .u128(),
            INIT_USER_BALANCE - FIRST_DEPOSIT_AMOUNT_ETH - SECOND_DEPOSIT_AMOUNT_ETH
        );

        assert_eq!(
            app.wrap()
                .query_balance(collateral_contract_addr.clone(), "eth")
                .unwrap()
                .amount
                .u128(),
            RESERVE_AMOUNT + FIRST_DEPOSIT_AMOUNT_ETH + SECOND_DEPOSIT_AMOUNT_ETH
        );
    }
}
