use cosmwasm_std::{coin, coins, Addr, BlockInfo, Timestamp};
use cw_multi_test::{App, BasicApp, ContractWrapper, Executor};
use std::vec;

use cosmwasm_std::Uint128;
use master_contract::msg::{
    ExecuteMsg,
    GetBalanceResponse,
    InstantiateMsg,
    QueryMsg,
};
use master_contract::{execute, instantiate, query};

pub fn success_deposit_of_one_token_setup() -> (BasicApp, Addr) {
    const DECIMAL_FRACTIONAL: Uint128 = Uint128::new(1_000_000_000_000_000_000u128); // 1*10**18

    const INIT_USER_BALANCE: u128 = 1000 * DECIMAL_FRACTIONAL.u128();
    const CONTRACT_RESERVES: u128 = 1000000 * DECIMAL_FRACTIONAL.u128();
    const FIRST_DEPOSIT_AMOUNT: u128 = 200 * DECIMAL_FRACTIONAL.u128();
    const SECOND_DEPOSIT_AMOUNT: u128 = 300 * DECIMAL_FRACTIONAL.u128();

    const MIN_INTEREST_RATE: u128 = 5u128 * DECIMAL_FRACTIONAL.u128();
    const SAFE_BORROW_MAX_RATE: u128 = 30u128 * DECIMAL_FRACTIONAL.u128();
    const RATE_GROWTH_FACTOR: u128 = 70u128 * DECIMAL_FRACTIONAL.u128();

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
    const DECIMAL_FRACTIONAL: Uint128 = Uint128::new(1_000_000_000_000_000_000u128); // 1*10**18

    const INIT_BALANCE_FIRST_TOKEN: u128 = 1000 * DECIMAL_FRACTIONAL.u128();
    const INIT_BALANCE_SECOND_TOKEN: u128 = 1000 * DECIMAL_FRACTIONAL.u128();

    const DEPOSIT_OF_FIRST_TOKEN: u128 = 200 * DECIMAL_FRACTIONAL.u128();
    const DEPOSIT_OF_SECOND_TOKEN: u128 = 300 * DECIMAL_FRACTIONAL.u128();

    const CONTRACT_RESERVES_FIRST_TOKEN: u128 = 1000 * DECIMAL_FRACTIONAL.u128();
    const CONTRACT_RESERVES_SECOND_TOKEN: u128 = 1000 * DECIMAL_FRACTIONAL.u128();

    const DECIMAL_FRACTIONAL_INT_RATE: Uint128 = Uint128::new(1_000_000_000_000_000_000u128); // 1*10**18

    const MIN_INTEREST_RATE: u128 = 5u128 * DECIMAL_FRACTIONAL_INT_RATE.u128();
    const SAFE_BORROW_MAX_RATE: u128 = 30u128 * DECIMAL_FRACTIONAL_INT_RATE.u128();
    const RATE_GROWTH_FACTOR: u128 = 70u128 * DECIMAL_FRACTIONAL_INT_RATE.u128();

    const PRICE_DECIMALS: u32 = 8;
    const PRICE_ETH: u128 = 2000u128 * 10u128.pow(PRICE_DECIMALS);
    const PRICE_ATOM: u128 = 10u128 * 10u128.pow(PRICE_DECIMALS);

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
            price: PRICE_ETH,
        },
        &[],
    )
    .unwrap();

    app.execute_contract(
        Addr::unchecked("owner"),
        addr.clone(),
        &ExecuteMsg::SetPrice {
            denom: "atom".to_string(),
            price: PRICE_ATOM,
        },
        &[],
    )
    .unwrap();

    let get_price_eth: Uint128 = app
        .wrap()
        .query_wasm_smart(
            addr.clone(),
            &QueryMsg::GetPrice {
                denom: "eth".to_string(),
            },
        )
        .unwrap();

    let get_price_atom: Uint128 = app
        .wrap()
        .query_wasm_smart(
            addr.clone(),
            &QueryMsg::GetPrice {
                denom: "atom".to_string(),
            },
        )
        .unwrap();

    assert_eq!(get_price_atom.u128(), 1000000000); // 10$
    assert_eq!(get_price_eth.u128(), 200000000000); // 2000$

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
    const DECIMAL_FRACTIONAL: Uint128 = Uint128::new(1_000_000_000_000_000_000u128); // 1*10**18

    const INIT_BALANCE_FIRST_TOKEN: u128 = 10000 * DECIMAL_FRACTIONAL.u128();
    const INIT_BALANCE_SECOND_TOKEN: u128 = 10000 * DECIMAL_FRACTIONAL.u128();

    const DEPOSIT_OF_FIRST_TOKEN: u128 = 200 * DECIMAL_FRACTIONAL.u128();
    const DEPOSIT_OF_SECOND_TOKEN: u128 = 300 * DECIMAL_FRACTIONAL.u128();

    const CONTRACT_RESERVES_FIRST_TOKEN: u128 = 1000 * DECIMAL_FRACTIONAL.u128();
    const CONTRACT_RESERVES_SECOND_TOKEN: u128 = 1000 * DECIMAL_FRACTIONAL.u128();

    const BORROW_OF_FIRST_TOKEN: u128 = 50 * DECIMAL_FRACTIONAL.u128();

    const MIN_INTEREST_RATE: u128 = 5u128 * DECIMAL_FRACTIONAL.u128();
    const SAFE_BORROW_MAX_RATE: u128 = 30u128 * DECIMAL_FRACTIONAL.u128();
    const RATE_GROWTH_FACTOR: u128 = 70u128 * DECIMAL_FRACTIONAL.u128();

    const PRICE_DECIMALS: u32 = 8;
    const PRICE_ETH: u128 = 2000u128 * 10u128.pow(PRICE_DECIMALS);
    const PRICE_ATOM: u128 = 10u128 * 10u128.pow(PRICE_DECIMALS);

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
                        6,
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
            price: PRICE_ETH,
        },
        &[],
    )
    .unwrap();

    app.execute_contract(
        Addr::unchecked("owner"),
        addr.clone(),
        &ExecuteMsg::SetPrice {
            denom: "atom".to_string(),
            price: PRICE_ATOM,
        },
        &[],
    )
    .unwrap();

    let get_price_eth: Uint128 = app
        .wrap()
        .query_wasm_smart(
            addr.clone(),
            &QueryMsg::GetPrice {
                denom: "eth".to_string(),
            },
        )
        .unwrap();

    let get_price_atom: Uint128 = app
        .wrap()
        .query_wasm_smart(
            addr.clone(),
            &QueryMsg::GetPrice {
                denom: "atom".to_string(),
            },
        )
        .unwrap();

    assert_eq!(get_price_atom.u128(), 1000000000); // 10$
    assert_eq!(get_price_eth.u128(), 200000000000); // 2000$

    app.set_block(BlockInfo {
        height: 0,
        time: Timestamp::from_seconds(0),
        chain_id: "custom_chain_id".to_string(),
    });

    app.execute_contract(
        Addr::unchecked("user"),
        addr.clone(),
        &ExecuteMsg::Deposit {},
        &coins(DEPOSIT_OF_FIRST_TOKEN, "eth"),
    )
    .unwrap();

    app.set_block(BlockInfo {
        height: 0,
        time: Timestamp::from_seconds(1000),
        chain_id: "custom_chain_id".to_string(),
    });

    let _available_to_redeem: Uint128 = app
        .wrap()
        .query_wasm_smart(
            addr.clone(),
            &QueryMsg::GetAvailableToRedeem {
                address: "user".to_string(),
                denom: "eth".to_string(),
            },
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

    app.set_block(BlockInfo {
        height: 0,
        time: Timestamp::from_seconds(2000),
        chain_id: "custom_chain_id".to_string(),
    });

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

    app.set_block(BlockInfo {
        height: 542,
        time: Timestamp::from_seconds(10000),
        chain_id: "custom_chain_id".to_string(),
    });

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

    (app, addr)
}
