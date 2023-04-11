#[cfg(test)]
mod tests {
    use cosmwasm_std::{coin, coins, Addr};
    use cw_multi_test::{App, ContractWrapper, Executor};
    use std::vec;

    use cosmwasm_std::Uint128;
    use master_contract::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
    use master_contract::{execute, instantiate, query};

    use crate::utils::success_deposit_setup;

    #[test]
    fn test_succes_withdraw_one_token() {
        const INIT_USER_BALANCE: u128 = 1000;
        const CONTRACT_RESERVES: u128 = 1000000;
        const FIRST_DEPOSIT_AMOUNT: u128 = 200;
        const SECOND_DEPOSIT_AMOUNT: u128 = 300;

        // having 500 deposited we want to withdraw SECOND_DEPOSIT_AMOUNT
        // so that FIRST_DEPOSIT_AMOUNT is remaining
        let (mut app, addr) = success_deposit_setup();

        app.execute_contract(
            Addr::unchecked("user"),
            addr.clone(),
            &ExecuteMsg::Withdraw {
                denom: "eth".to_string(),
                amount: Uint128::from(SECOND_DEPOSIT_AMOUNT),
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

        assert_eq!(user_deposited_balance.u128(), FIRST_DEPOSIT_AMOUNT);

        assert_eq!(
            app.wrap()
                .query_balance("user", "eth")
                .unwrap()
                .amount
                .u128(),
            INIT_USER_BALANCE - FIRST_DEPOSIT_AMOUNT
        );
    }
}
