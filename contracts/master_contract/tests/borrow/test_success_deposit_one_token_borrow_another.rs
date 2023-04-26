#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::{
        success_deposit_of_diff_token_with_prices, success_deposit_of_one_token_setup,
    };
    use cosmwasm_std::{coin, coins, Addr, Uint128};
    use cw_multi_test::{App, ContractWrapper, Executor};
    use master_contract::msg::{
        ExecuteMsg, GetBalanceResponse, GetBorrowsResponse, GetPriceResponse, InstantiateMsg,
        QueryMsg,
    };
    use master_contract::{execute, instantiate, query};

    #[test]
    fn test_success_deposit_one_token_borrow_another() {
        const INIT_BALANCE_FIRST_TOKEN: u128 = 1000;
        const INIT_BALANCE_SECOND_TOKEN: u128 = 1000;

        const DEPOSIT_OF_FIRST_TOKEN: u128 = 200;

        const CONTRACT_RESERVES_FIRST_TOKEN: u128 = 1000;
        const CONTRACT_RESERVES_SECOND_TOKEN: u128 = 1000;

        const BORROW_SECOND_TOKEN: u128 = 300;

        let mut app = App::new(|router, _, storage| {
            router
                .bank
                .init_balance(
                    storage,
                    &Addr::unchecked("user"),
                    vec![
                        coin(INIT_BALANCE_FIRST_TOKEN, "eth"),
                        coin(INIT_BALANCE_SECOND_TOKEN, "atom"),
                    ],
                )
                .unwrap();

            router
                .bank
                .init_balance(
                    storage,
                    &Addr::unchecked("owner"),
                    vec![
                        coin(CONTRACT_RESERVES_FIRST_TOKEN, "eth"),
                        coin(CONTRACT_RESERVES_SECOND_TOKEN, "atom"),
                    ],
                )
                .unwrap();
        });

        let code = ContractWrapper::new(execute, instantiate, query);
        let code_id = app.store_code(Box::new(code));

        let addr = app
            .instantiate_contract(
                code_id,
                Addr::unchecked("owner"),
                &InstantiateMsg {
                    admin: "owner".to_string(),
                    supported_tokens: vec![
                        (
                            "eth".to_string(),
                            "ethereum".to_string(),
                            "ETH".to_string(),
                            18,
                        ),
                        (
                            "atom".to_string(),
                            "atom".to_string(),
                            "ATOM".to_string(),
                            18,
                        ),
                    ],
                },
                &[coin(CONTRACT_RESERVES_SECOND_TOKEN, "atom")],
                "Contract",
                Some("owner".to_string()), // contract that can execute migrations
            )
            .unwrap();

        app.execute_contract(
            Addr::unchecked("owner"),
            addr.clone(),
            &ExecuteMsg::SetPrice {
                denom: "eth".to_string(),
                price: 2000,
            },
            &[],
        )
            .unwrap();

        app.execute_contract(
            Addr::unchecked("owner"),
            addr.clone(),
            &ExecuteMsg::SetPrice {
                denom: "atom".to_string(),
                price: 10,
            },
            &[],
        )
            .unwrap();

        let get_price_eth: GetPriceResponse = app
            .wrap()
            .query_wasm_smart(
                addr.clone(),
                &QueryMsg::GetPrice {
                    denom: "eth".to_string(),
                },
            )
            .unwrap();

        let get_price_atom: GetPriceResponse = app
            .wrap()
            .query_wasm_smart(
                addr.clone(),
                &QueryMsg::GetPrice {
                    denom: "atom".to_string(),
                },
            )
            .unwrap();

        assert_eq!(get_price_atom.price, 10);

        assert_eq!(get_price_eth.price, 2000);

        // funding contract with second reserve
        app.execute_contract(
            Addr::unchecked("owner"),
            addr.clone(),
            &ExecuteMsg::Fund {},
            &coins(CONTRACT_RESERVES_FIRST_TOKEN, "eth"),
        )
            .unwrap();

        app.execute_contract(
            Addr::unchecked("user"),
            addr.clone(),
            &ExecuteMsg::Deposit {},
            &coins(DEPOSIT_OF_FIRST_TOKEN, "eth"),
        )
            .unwrap();


        let user_available_to_borrow_another_token: Uint128 = app
            .wrap()
            .query_wasm_smart(
                addr.clone(),
                &QueryMsg::GetAvailableToBorrow {
                    address: "user".to_string(),
                    denom: "atom".to_string(),
                },
            )
            .unwrap();

        assert_eq!(user_available_to_borrow_another_token.u128(), DEPOSIT_OF_FIRST_TOKEN * get_price_eth.price * 8 / 10 / get_price_atom.price);

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

        assert_eq!(
            user_deposited_balance.balance.u128(),
            DEPOSIT_OF_FIRST_TOKEN
        );

        assert_eq!(
            app.wrap()
                .query_balance("user", "eth")
                .unwrap()
                .amount
                .u128(),
            INIT_BALANCE_FIRST_TOKEN - DEPOSIT_OF_FIRST_TOKEN
        );

        assert_eq!(
            app.wrap()
                .query_balance(&addr, "eth")
                .unwrap()
                .amount
                .u128(),
            CONTRACT_RESERVES_FIRST_TOKEN + DEPOSIT_OF_FIRST_TOKEN
        );

        app.execute_contract(
            Addr::unchecked("user"),
            addr.clone(),
            &ExecuteMsg::Borrow {
                denom: "atom".to_string(),
                amount: Uint128::from(BORROW_SECOND_TOKEN),
            },
            &[],
        )
            .unwrap();

        let user_borrowed_balance: GetBorrowsResponse = app
            .wrap()
            .query_wasm_smart(
                addr.clone(),
                &QueryMsg::GetBorrows {
                    address: "user".to_string(),
                    denom: "atom".to_string(),
                },
            )
            .unwrap();

        assert_eq!(user_borrowed_balance.borrows.u128(), BORROW_SECOND_TOKEN);
    }
}
