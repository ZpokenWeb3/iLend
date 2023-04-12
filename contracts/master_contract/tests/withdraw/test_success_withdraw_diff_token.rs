#[cfg(test)]
mod tests {
    use cosmwasm_std::{Addr};
    use cw_multi_test::{Executor};


    use cosmwasm_std::Uint128;
    use master_contract::msg::{ExecuteMsg, QueryMsg};
    use crate::utils::success_deposit_of_diff_token_setup;

    #[test]
    fn test_success_withdraw_diff_token() {

        const INIT_BALANCE_FIRST_TOKEN: u128 = 1000;
        const INIT_BALANCE_SECOND_TOKEN: u128 = 1000;

        const DEPOSIT_OF_FIRST_TOKEN: u128 = 200;
        const DEPOSIT_OF_SECOND_TOKEN: u128 = 300;

        const CONTRACT_RESERVES_FIRST_TOKEN: u128 = 1000;
        const CONTRACT_RESERVES_SECOND_TOKEN: u128 = 1000;

        const WITHDRAW_AMOUNT_FIRST_TOKEN: u128 = 100;
        const WITHDRAW_AMOUNT_SECOND_TOKEN: u128 = 150;


        let (mut app, addr) = success_deposit_of_diff_token_setup();

        app.execute_contract(
            Addr::unchecked("user"),
            addr.clone(),
            &ExecuteMsg::Withdraw {
                denom: "eth".to_string(),
                amount: Uint128::from(WITHDRAW_AMOUNT_FIRST_TOKEN),
            },
            &[],
        )
            .unwrap();

        app.execute_contract(
            Addr::unchecked("user"),
            addr.clone(),
            &ExecuteMsg::Withdraw {
                denom: "atom".to_string(),
                amount: Uint128::from(WITHDRAW_AMOUNT_SECOND_TOKEN),
            },
            &[],
        )
            .unwrap();

        let user_deposited_balance_of_first_token: Uint128 = app
            .wrap()
            .query_wasm_smart(
                addr.clone(),
                &QueryMsg::GetDeposit {
                    address: "user".to_string(),
                    denom: "eth".to_string(),
                },
            )
            .unwrap();

        assert_eq!(user_deposited_balance_of_first_token.u128(), DEPOSIT_OF_FIRST_TOKEN - WITHDRAW_AMOUNT_FIRST_TOKEN);

        assert_eq!(
            app.wrap()
                .query_balance("user", "eth")
                .unwrap()
                .amount
                .u128(),
            INIT_BALANCE_FIRST_TOKEN - DEPOSIT_OF_FIRST_TOKEN + WITHDRAW_AMOUNT_FIRST_TOKEN
        );


        let user_deposited_balance_of_second_token: Uint128 = app
            .wrap()
            .query_wasm_smart(
                addr.clone(),
                &QueryMsg::GetDeposit {
                    address: "user".to_string(),
                    denom: "atom".to_string(),
                },
            )
            .unwrap();

        assert_eq!(user_deposited_balance_of_second_token.u128(), DEPOSIT_OF_SECOND_TOKEN - WITHDRAW_AMOUNT_SECOND_TOKEN);

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