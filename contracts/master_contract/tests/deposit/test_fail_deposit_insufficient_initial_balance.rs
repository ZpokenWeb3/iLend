#[cfg(test)]
mod tests {
    use cosmwasm_std::{coin, coins, Addr};
    use cw_multi_test::{App, ContractWrapper, Executor};
    use std::vec;

    use cosmwasm_std::Uint128;
    use master_contract::msg::{ExecuteMsg, GetBalanceResponse, InstantiateMsg, QueryMsg};
    use master_contract::{execute, instantiate, query};
    use pyth_sdk_cw::PriceIdentifier;

    #[test]
    fn test_fail_deposit_insufficient_initial_balance() {
        const INIT_USER_BALANCE: u128 = 1000;
        const CONTRACT_RESERVES: u128 = 1000000;
        const FIRST_DEPOSIT_AMOUNT: u128 = 2000;

        let mut app = App::new(|router, _, storage| {
            router
                .bank
                .init_balance(
                    storage,
                    &Addr::unchecked("user"),
                    coins(INIT_USER_BALANCE, "eth"),
                )
                .unwrap();

            router
                .bank
                .init_balance(
                    storage,
                    &Addr::unchecked("owner"),
                    coins(CONTRACT_RESERVES, "eth"),
                )
                .unwrap()
        });

        let code = ContractWrapper::new(execute, instantiate, query);
        let code_id = app.store_code(Box::new(code));

        let addr = app
            .instantiate_contract(
                code_id,
                Addr::unchecked("owner"),
                &InstantiateMsg {
                    admin: "owner".to_string(),
                    supported_tokens: vec![("eth".to_string(), "ieth".to_string())],
                },
                &[coin(CONTRACT_RESERVES, "eth")],
                "Contract",
                Some("owner".to_string()), // contract that can execute migrations
            )
            .unwrap();

        assert!(app
            .execute_contract(
                Addr::unchecked("user"),
                addr.clone(),
                &ExecuteMsg::Deposit {},
                &coins(FIRST_DEPOSIT_AMOUNT, "eth"),
            )
            .is_err());

        let user_deposited_balance: GetBalanceResponse = app
            .wrap()
            .query_wasm_smart(
                addr.clone(),
                &QueryMsg::GetDeposit {
                    address: "user".to_string(),
                    denom: "eth".to_string(),
                },
            )
            .unwrap();

        // there is no deposit executed, so have to be zero
        assert_eq!(user_deposited_balance.balance.u128(), 0);
    }
}
