#[cfg(test)]
mod tests {
    use crate::utils::success_collateral_margin_setup_with_deposit;
    use cosmwasm_std::{coins, Addr, Uint128};
    use cw_multi_test::Executor;

    use margin_positions::msg::{
        ExecuteMsg as ExecuteMsgMarginPositions, QueryMsg as QueryMsgMarginPositions,
    };

    #[test]
    fn test_success_redeem_diff_token() {
        const TOKENS_DECIMALS: u32 = 18;
        const INIT_USER_BALANCE: u128 = 1000 * 10u128.pow(TOKENS_DECIMALS);
        const RESERVE_AMOUNT: u128 = 1000 * 10u128.pow(TOKENS_DECIMALS);
        const FIRST_DEPOSIT_AMOUNT_ETH: u128 = 200 * 10u128.pow(TOKENS_DECIMALS);
        const SECOND_DEPOSIT_AMOUNT_ETH: u128 = 300 * 10u128.pow(TOKENS_DECIMALS);

        const DEPOSIT_AMOUNT_ATOM: u128 = 200 * 10u128.pow(TOKENS_DECIMALS);

        let (mut app, margin_positions_addr, collateral_contract_addr) =
            success_collateral_margin_setup_with_deposit();

        app.execute_contract(
            Addr::unchecked("user"),
            margin_positions_addr.clone(),
            &ExecuteMsgMarginPositions::Deposit {},
            &coins(DEPOSIT_AMOUNT_ATOM, "atom"),
        )
        .unwrap();

        let user_deposited_balance: Uint128 = app
            .wrap()
            .query_wasm_smart(
                margin_positions_addr.clone(),
                &QueryMsgMarginPositions::GetDeposit {
                    address: "user".to_string(),
                    denom: "atom".to_string(),
                },
            )
            .unwrap();

        assert_eq!(user_deposited_balance.u128(), DEPOSIT_AMOUNT_ATOM);

        assert_eq!(
            app.wrap()
                .query_balance("user", "atom")
                .unwrap()
                .amount
                .u128(),
            INIT_USER_BALANCE - DEPOSIT_AMOUNT_ATOM
        );

        assert_eq!(
            app.wrap()
                .query_balance(collateral_contract_addr.clone(), "atom")
                .unwrap()
                .amount
                .u128(),
            RESERVE_AMOUNT + DEPOSIT_AMOUNT_ATOM
        );

        app.execute_contract(
            Addr::unchecked("user"),
            margin_positions_addr.clone(),
            &ExecuteMsgMarginPositions::Redeem {
                denom: "eth".to_string(),
                amount: Uint128::from(SECOND_DEPOSIT_AMOUNT_ETH),
            },
            &[],
        )
        .unwrap();

        let user_deposited_balance_after_first_withdrawal: Uint128 = app
            .wrap()
            .query_wasm_smart(
                margin_positions_addr.clone(),
                &QueryMsgMarginPositions::GetDeposit {
                    address: "user".to_string(),
                    denom: "eth".to_string(),
                },
            )
            .unwrap();

        assert_eq!(
            user_deposited_balance_after_first_withdrawal.u128(),
            FIRST_DEPOSIT_AMOUNT_ETH
        );

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
            &ExecuteMsgMarginPositions::Redeem {
                denom: "atom".to_string(),
                amount: Uint128::from(DEPOSIT_AMOUNT_ATOM),
            },
            &[],
        )
        .unwrap();

        let user_deposited_balance_after_withdrawal: Uint128 = app
            .wrap()
            .query_wasm_smart(
                margin_positions_addr.clone(),
                &QueryMsgMarginPositions::GetDeposit {
                    address: "user".to_string(),
                    denom: "atom".to_string(),
                },
            )
            .unwrap();

        assert_eq!(user_deposited_balance_after_withdrawal.u128(), 0);

        assert_eq!(
            app.wrap()
                .query_balance("user", "atom")
                .unwrap()
                .amount
                .u128(),
            INIT_USER_BALANCE
        );

        assert_eq!(
            app.wrap()
                .query_balance(collateral_contract_addr.clone(), "atom")
                .unwrap()
                .amount
                .u128(),
            RESERVE_AMOUNT
        );
    }
}
