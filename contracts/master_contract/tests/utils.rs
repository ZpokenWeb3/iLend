use cosmwasm_std::{coin, coins, Addr};
use cw_multi_test::{App, BasicApp, ContractWrapper, Executor};
use std::vec;

use cosmwasm_std::Uint128;
use master_contract::msg::{
    ExecuteMsg, GetBalanceResponse, GetBorrowsResponse, GetPriceResponse, InstantiateMsg, QueryMsg,
};
use master_contract::{execute, instantiate, query};

pub fn success_deposit_of_one_token_setup() -> (BasicApp, Addr) {
    const INIT_USER_BALANCE: u128 = 1000;
    const CONTRACT_RESERVES: u128 = 1000000;
    const FIRST_DEPOSIT_AMOUNT: u128 = 200;
    const SECOND_DEPOSIT_AMOUNT: u128 = 300;

    const INTEREST_RATE_DECIMALS: u32 = 18;

    const MIN_INTEREST_RATE: u128 = 5u128 * 10u128.pow(INTEREST_RATE_DECIMALS);
    const SAFE_BORROW_MAX_RATE: u128 = 30u128 * 10u128.pow(INTEREST_RATE_DECIMALS);
    const RATE_GROWTH_FACTOR: u128 = 70u128 * 10u128.pow(INTEREST_RATE_DECIMALS);

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
                tokens_interest_rate_model_params: vec![
                  (
                      "eth".to_string(),
                      MIN_INTEREST_RATE,
                      SAFE_BORROW_MAX_RATE,
                      RATE_GROWTH_FACTOR,
                  ),
                  (
                      "atom".to_string(),
                      MIN_INTEREST_RATE,
                      SAFE_BORROW_MAX_RATE,
                      RATE_GROWTH_FACTOR,
                  ),
                ],
            },
            &[coin(CONTRACT_RESERVES, "eth")],
            "Contract",
            Some("owner".to_string()), // contract that can execute migrations
        )
        .unwrap();

    app.execute_contract(
        Addr::unchecked("user"),
        addr.clone(),
        &ExecuteMsg::Deposit {},
        &coins(FIRST_DEPOSIT_AMOUNT, "eth"),
    )
    .unwrap();

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

    assert_eq!(user_deposited_balance.balance.u128(), FIRST_DEPOSIT_AMOUNT);

    assert_eq!(
        app.wrap()
            .query_balance("user", "eth")
            .unwrap()
            .amount
            .u128(),
        INIT_USER_BALANCE - FIRST_DEPOSIT_AMOUNT
    );

    assert_eq!(
        app.wrap()
            .query_balance(&addr, "eth")
            .unwrap()
            .amount
            .u128(),
        CONTRACT_RESERVES + FIRST_DEPOSIT_AMOUNT
    );

    app.execute_contract(
        Addr::unchecked("user"),
        addr.clone(),
        &ExecuteMsg::Deposit {},
        &coins(SECOND_DEPOSIT_AMOUNT, "eth"),
    )
    .unwrap();

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
        FIRST_DEPOSIT_AMOUNT + SECOND_DEPOSIT_AMOUNT
    );

    assert_eq!(
        app.wrap()
            .query_balance("user", "eth")
            .unwrap()
            .amount
            .u128(),
        INIT_USER_BALANCE - FIRST_DEPOSIT_AMOUNT - SECOND_DEPOSIT_AMOUNT
    );

    assert_eq!(
        app.wrap()
            .query_balance(&addr, "eth")
            .unwrap()
            .amount
            .u128(),
        CONTRACT_RESERVES + FIRST_DEPOSIT_AMOUNT + SECOND_DEPOSIT_AMOUNT
    );

    (app, addr)
}

pub fn success_deposit_of_diff_token_with_prices() -> (BasicApp, Addr) {
    const INIT_BALANCE_FIRST_TOKEN: u128 = 1000;
    const INIT_BALANCE_SECOND_TOKEN: u128 = 1000;

    const DEPOSIT_OF_FIRST_TOKEN: u128 = 200;
    const DEPOSIT_OF_SECOND_TOKEN: u128 = 300;

    const CONTRACT_RESERVES_FIRST_TOKEN: u128 = 1000;
    const CONTRACT_RESERVES_SECOND_TOKEN: u128 = 1000;

    const INTEREST_RATE_DECIMALS: u32 = 18;

    const MIN_INTEREST_RATE: u128 = 5u128 * 10u128.pow(INTEREST_RATE_DECIMALS);
    const SAFE_BORROW_MAX_RATE: u128 = 30u128 * 10u128.pow(INTEREST_RATE_DECIMALS);
    const RATE_GROWTH_FACTOR: u128 = 70u128 * 10u128.pow(INTEREST_RATE_DECIMALS);

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
                tokens_interest_rate_model_params: vec![
                  (
                      "eth".to_string(),
                      MIN_INTEREST_RATE,
                      SAFE_BORROW_MAX_RATE,
                      RATE_GROWTH_FACTOR,
                  ),
                  (
                      "atom".to_string(),
                      MIN_INTEREST_RATE,
                      SAFE_BORROW_MAX_RATE,
                      RATE_GROWTH_FACTOR,
                  ),
                ],
            },
            &[coin(CONTRACT_RESERVES_SECOND_TOKEN, "atom")],
            "Contract",
            Some("owner".to_string()), // contract that can execute migrations
        )
        .unwrap();

    // funding contract with second reserve
    app.execute_contract(
        Addr::unchecked("owner"),
        addr.clone(),
        &ExecuteMsg::Fund {},
        &coins(CONTRACT_RESERVES_FIRST_TOKEN, "eth"),
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

    app.execute_contract(
        Addr::unchecked("user"),
        addr.clone(),
        &ExecuteMsg::Deposit {},
        &coins(DEPOSIT_OF_FIRST_TOKEN, "eth"),
    )
    .unwrap();

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
        &ExecuteMsg::Deposit {},
        &coins(DEPOSIT_OF_SECOND_TOKEN, "atom"),
    )
    .unwrap();

    let user_deposited_balance: GetBalanceResponse = app
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
        user_deposited_balance.balance.u128(),
        DEPOSIT_OF_SECOND_TOKEN
    );

    assert_eq!(
        app.wrap()
            .query_balance("user", "atom")
            .unwrap()
            .amount
            .u128(),
        INIT_BALANCE_SECOND_TOKEN - DEPOSIT_OF_SECOND_TOKEN
    );

    assert_eq!(
        app.wrap()
            .query_balance(&addr, "atom")
            .unwrap()
            .amount
            .u128(),
        CONTRACT_RESERVES_SECOND_TOKEN + DEPOSIT_OF_SECOND_TOKEN
    );

    (app, addr)
}

pub fn success_borrow_setup() -> (BasicApp, Addr) {
    const TOKEN_DECIMAL: u128 = 10u128.pow(18) as u128;

    const INIT_BALANCE_FIRST_TOKEN: u128 = 1000 * TOKEN_DECIMAL;
    const INIT_BALANCE_SECOND_TOKEN: u128 = 1000 * TOKEN_DECIMAL;

    const DEPOSIT_OF_FIRST_TOKEN: u128 = 200 * TOKEN_DECIMAL;
    const DEPOSIT_OF_SECOND_TOKEN: u128 = 300 * TOKEN_DECIMAL;

    const CONTRACT_RESERVES_FIRST_TOKEN: u128 = 1000 * TOKEN_DECIMAL;
    const CONTRACT_RESERVES_SECOND_TOKEN: u128 = 1000 * TOKEN_DECIMAL;

    const BORROW_OF_FIRST_TOKEN: u128 = 50 * TOKEN_DECIMAL;

    const INTEREST_RATE_DECIMALS: u32 = 18;

    const MIN_INTEREST_RATE: u128 = 5u128 * 10u128.pow(INTEREST_RATE_DECIMALS);
    const SAFE_BORROW_MAX_RATE: u128 = 30u128 * 10u128.pow(INTEREST_RATE_DECIMALS);
    const RATE_GROWTH_FACTOR: u128 = 70u128 * 10u128.pow(INTEREST_RATE_DECIMALS);

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
                tokens_interest_rate_model_params: vec![
                  (
                      "eth".to_string(),
                      MIN_INTEREST_RATE,
                      SAFE_BORROW_MAX_RATE,
                      RATE_GROWTH_FACTOR,
                  ),
                  (
                      "atom".to_string(),
                      MIN_INTEREST_RATE,
                      SAFE_BORROW_MAX_RATE,
                      RATE_GROWTH_FACTOR,
                  ),
                ],
            },
            &[coin(CONTRACT_RESERVES_SECOND_TOKEN, "atom")],
            "Contract",
            Some("owner".to_string()), // contract that can execute migrations
        )
        .unwrap();

    // funding contract with second reserve
    app.execute_contract(
        Addr::unchecked("owner"),
        addr.clone(),
        &ExecuteMsg::Fund {},
        &coins(CONTRACT_RESERVES_FIRST_TOKEN, "eth"),
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

    app.execute_contract(
        Addr::unchecked("user"),
        addr.clone(),
        &ExecuteMsg::Deposit {},
        &coins(DEPOSIT_OF_FIRST_TOKEN, "eth"),
    )
    .unwrap();

    let available_to_redeem: Uint128 = app
        .wrap()
        .query_wasm_smart(
            addr.clone(),
            &QueryMsg::GetAvailableToRedeem {
                address: "user".to_string(),
                denom: "eth".to_string(),
            },
        )
        .unwrap();

    println!("{}", available_to_redeem.u128());

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
        &ExecuteMsg::Deposit {},
        &coins(DEPOSIT_OF_SECOND_TOKEN, "atom"),
    )
    .unwrap();

    let user_deposited_balance: GetBalanceResponse = app
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
        user_deposited_balance.balance.u128(),
        DEPOSIT_OF_SECOND_TOKEN
    );

    assert_eq!(
        app.wrap()
            .query_balance("user", "atom")
            .unwrap()
            .amount
            .u128(),
        INIT_BALANCE_SECOND_TOKEN - DEPOSIT_OF_SECOND_TOKEN
    );

    assert_eq!(
        app.wrap()
            .query_balance(&addr, "atom")
            .unwrap()
            .amount
            .u128(),
        CONTRACT_RESERVES_SECOND_TOKEN + DEPOSIT_OF_SECOND_TOKEN
    );

    app.execute_contract(
        Addr::unchecked("user"),
        addr.clone(),
        &ExecuteMsg::Borrow {
            denom: "eth".to_string(),
            amount: Uint128::from(BORROW_OF_FIRST_TOKEN),
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
                denom: "eth".to_string(),
            },
        )
        .unwrap();

    assert_eq!(user_borrowed_balance.borrows.u128(), BORROW_OF_FIRST_TOKEN);

    (app, addr)
}
