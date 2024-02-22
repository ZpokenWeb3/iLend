use cosmwasm_std::{coin, coins, Addr, BlockInfo, Timestamp};
use cw_multi_test::{App, BasicApp, ContractWrapper, Executor};
use std::vec;

use cosmwasm_std::Uint128;
use cw20_base::contract::{
    execute as execute_cw20, instantiate as instantiate_cw20, query as query_cw20,
};
use cw20_base::msg::{
    ExecuteMsg as ExecuteMsgCW20, InstantiateMsg as InstantiateMsgCW20, QueryMsg as QueryMsgCW20,
};
use lending::msg::{ExecuteMsg, GetBalanceResponse, InstantiateMsg, QueryMsg};
use lending::{execute, instantiate, query};

use cw20::{BalanceResponse, Cw20Coin, Cw20QueryMsg, MinterResponse};
use pyth_sdk_cw::PriceIdentifier;
use std::time::{SystemTime, UNIX_EPOCH};

pub fn success_deposit_of_one_token_setup() -> (BasicApp, Addr) {
    const TOKENS_DECIMALS: u32 = 18;

    const INIT_USER_BALANCE: u128 = 1000 * 10u128.pow(TOKENS_DECIMALS);
    const INIT_LIQUIDATOR_BALANCE_ETH: u128 = 1_000_000 * 10u128.pow(TOKENS_DECIMALS); // 1M ETH

    const CONTRACT_RESERVES: u128 = 1000000 * 10u128.pow(TOKENS_DECIMALS);
    const FIRST_DEPOSIT_AMOUNT_ETH: u128 = 200 * 10u128.pow(TOKENS_DECIMALS);
    const SECOND_DEPOSIT_AMOUNT_ETH: u128 = 300 * 10u128.pow(TOKENS_DECIMALS);

    const PERCENT_DECIMALS: u32 = 5;
    const LTV_ETH: u128 = 85 * 10u128.pow(PERCENT_DECIMALS); // 85%
    const LIQUIDATION_THRESHOLD_ETH: u128 = 90 * 10u128.pow(PERCENT_DECIMALS); // 90%
    const LTV_ATOM: u128 = 75 * 10u128.pow(PERCENT_DECIMALS); // 75%
    const LIQUIDATION_THRESHOLD_ATOM: u128 = 80 * 10u128.pow(PERCENT_DECIMALS); // 80%

    const INTEREST_RATE_DECIMALS: u32 = 18;
    const MIN_INTEREST_RATE: u128 = 5 * 10u128.pow(INTEREST_RATE_DECIMALS);
    const SAFE_BORROW_MAX_RATE: u128 = 30 * 10u128.pow(INTEREST_RATE_DECIMALS);
    const RATE_GROWTH_FACTOR: u128 = 70 * 10u128.pow(INTEREST_RATE_DECIMALS);

    const OPTIMAL_UTILISATION_RATIO: u128 = 80 * 10u128.pow(PERCENT_DECIMALS);

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
            .unwrap();

        router
            .bank
            .init_balance(
                storage,
                &Addr::unchecked("liquidator"),
                coins(INIT_LIQUIDATOR_BALANCE_ETH, "eth"),
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
                is_testing: true,
                price_ids: vec![
                    (
                        "inj".to_string(),
                        PriceIdentifier::from_hex(
                            "2d9315a88f3019f8efa88dfe9c0f0843712da0bac814461e27733f6b83eb51b3",
                        )
                        .unwrap(),
                    ),
                    (
                        "peggy0x44C21afAaF20c270EBbF5914Cfc3b5022173FEB7".to_string(),
                        PriceIdentifier::from_hex(
                            "2d9315a88f3019f8efa88dfe9c0f0843712da0bac814461e27733f6b83eb51b3",
                        )
                        .unwrap(),
                    ),
                ],
                pyth_contract_addr: "inj1z60tg0tekdzcasenhuuwq3htjcd5slmgf7gpez".to_string(),
                admin: "owner".to_string(),
                supported_tokens: vec![
                    (
                        "eth".to_string(),
                        "ethereum".to_string(),
                        "ETH".to_string(),
                        None,
                        TOKENS_DECIMALS as u128,
                    ),
                    (
                        "atom".to_string(),
                        "atom".to_string(),
                        "ATOM".to_string(),
                        None,
                        TOKENS_DECIMALS as u128,
                    ),
                ],
                reserve_configuration: vec![
                    ("eth".to_string(), LTV_ETH, LIQUIDATION_THRESHOLD_ETH),
                    ("atom".to_string(), LTV_ATOM, LIQUIDATION_THRESHOLD_ATOM),
                ],
                tokens_interest_rate_model_params: vec![
                    (
                        "eth".to_string(),
                        MIN_INTEREST_RATE,
                        SAFE_BORROW_MAX_RATE,
                        RATE_GROWTH_FACTOR,
                        OPTIMAL_UTILISATION_RATIO,
                    ),
                    (
                        "atom".to_string(),
                        MIN_INTEREST_RATE,
                        SAFE_BORROW_MAX_RATE,
                        RATE_GROWTH_FACTOR,
                        OPTIMAL_UTILISATION_RATIO,
                    ),
                ],
                price_updater_addr: "".to_string(),
            },
            &[],
            "Contract",
            Some("owner".to_string()), // contract that can execute migrations
        )
        .unwrap();

    app.send_tokens(
        Addr::unchecked("owner"),
        addr.clone(),
        &[coin(CONTRACT_RESERVES, "eth")],
    )
    .unwrap();

    app.execute_contract(
        Addr::unchecked("user"),
        addr.clone(),
        &ExecuteMsg::Deposit {},
        &coins(FIRST_DEPOSIT_AMOUNT_ETH, "eth"),
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
            .query_balance(&addr, "eth")
            .unwrap()
            .amount
            .u128(),
        CONTRACT_RESERVES + FIRST_DEPOSIT_AMOUNT_ETH
    );

    app.execute_contract(
        Addr::unchecked("user"),
        addr.clone(),
        &ExecuteMsg::Deposit {},
        &coins(SECOND_DEPOSIT_AMOUNT_ETH, "eth"),
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
            .query_balance(&addr, "eth")
            .unwrap()
            .amount
            .u128(),
        CONTRACT_RESERVES + FIRST_DEPOSIT_AMOUNT_ETH + SECOND_DEPOSIT_AMOUNT_ETH
    );

    (app, addr)
}

pub fn success_deposit_of_diff_token_with_prices() -> (BasicApp, Addr) {
    const TOKENS_DECIMALS: u32 = 18;

    const INIT_BALANCE_ETH: u128 = 1000 * 10u128.pow(TOKENS_DECIMALS);
    const INIT_BALANCE_ATOM: u128 = 1_000_000 * 10u128.pow(TOKENS_DECIMALS); // 1M ATOM

    const INIT_LIQUIDATOR_BALANCE_ETH: u128 = 1_000_000 * 10u128.pow(TOKENS_DECIMALS); // 1M ETH
    const INIT_LIQUIDATOR_BALANCE_ATOM: u128 = 1_000_000 * 10u128.pow(TOKENS_DECIMALS); // 1M ATOM

    const DEPOSIT_AMOUNT_ETH: u128 = 200 * 10u128.pow(TOKENS_DECIMALS);
    const DEPOSIT_AMOUNT_ATOM: u128 = 300 * 10u128.pow(TOKENS_DECIMALS);

    const CONTRACT_RESERVES_ETH: u128 = 1000 * 10u128.pow(TOKENS_DECIMALS);
    const CONTRACT_RESERVES_ATOM: u128 = 1000 * 10u128.pow(TOKENS_DECIMALS);

    const PERCENT_DECIMALS: u32 = 5;
    const LTV_ETH: u128 = 85 * 10u128.pow(PERCENT_DECIMALS); // 85%
    const LIQUIDATION_THRESHOLD_ETH: u128 = 90 * 10u128.pow(PERCENT_DECIMALS); // 90%
    const LTV_ATOM: u128 = 75 * 10u128.pow(PERCENT_DECIMALS); // 75%
    const LIQUIDATION_THRESHOLD_ATOM: u128 = 80 * 10u128.pow(PERCENT_DECIMALS); // 80%

    const INTEREST_RATE_DECIMALS: u32 = 18;
    const MIN_INTEREST_RATE: u128 = 5 * 10u128.pow(INTEREST_RATE_DECIMALS);
    const SAFE_BORROW_MAX_RATE: u128 = 30 * 10u128.pow(INTEREST_RATE_DECIMALS);
    const RATE_GROWTH_FACTOR: u128 = 70 * 10u128.pow(INTEREST_RATE_DECIMALS);

    const OPTIMAL_UTILISATION_RATIO: u128 = 80 * 10u128.pow(PERCENT_DECIMALS);

    const PRICE_DECIMALS: u32 = 8;
    const PRICE_ETH: u128 = 2000 * 10u128.pow(PRICE_DECIMALS);
    const PRICE_ATOM: u128 = 10 * 10u128.pow(PRICE_DECIMALS);

    let mut app = App::new(|router, _, storage| {
        router
            .bank
            .init_balance(
                storage,
                &Addr::unchecked("user"),
                vec![
                    coin(INIT_BALANCE_ETH, "eth"),
                    coin(INIT_BALANCE_ATOM, "atom"),
                ],
            )
            .unwrap();

        router
            .bank
            .init_balance(
                storage,
                &Addr::unchecked("owner"),
                vec![
                    coin(INIT_BALANCE_ETH, "eth"),
                    coin(INIT_BALANCE_ATOM, "atom"),
                ],
            )
            .unwrap();

        router
            .bank
            .init_balance(
                storage,
                &Addr::unchecked("liquidator"),
                vec![
                    coin(INIT_LIQUIDATOR_BALANCE_ETH, "eth"),
                    coin(INIT_LIQUIDATOR_BALANCE_ATOM, "atom"),
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
                is_testing: true,
                admin: "owner".to_string(),
                supported_tokens: vec![
                    (
                        "eth".to_string(),
                        "ethereum".to_string(),
                        "ETH".to_string(),
                        None,
                        TOKENS_DECIMALS as u128,
                    ),
                    (
                        "atom".to_string(),
                        "atom".to_string(),
                        "ATOM".to_string(),
                        None,
                        TOKENS_DECIMALS as u128,
                    ),
                ],
                reserve_configuration: vec![
                    ("eth".to_string(), LTV_ETH, LIQUIDATION_THRESHOLD_ETH),
                    ("atom".to_string(), LTV_ATOM, LIQUIDATION_THRESHOLD_ATOM),
                ],
                tokens_interest_rate_model_params: vec![
                    (
                        "eth".to_string(),
                        MIN_INTEREST_RATE,
                        SAFE_BORROW_MAX_RATE,
                        RATE_GROWTH_FACTOR,
                        OPTIMAL_UTILISATION_RATIO,
                    ),
                    (
                        "atom".to_string(),
                        MIN_INTEREST_RATE,
                        SAFE_BORROW_MAX_RATE,
                        RATE_GROWTH_FACTOR,
                        OPTIMAL_UTILISATION_RATIO,
                    ),
                ],
                price_ids: vec![
                    (
                        "inj".to_string(),
                        PriceIdentifier::from_hex(
                            "2d9315a88f3019f8efa88dfe9c0f0843712da0bac814461e27733f6b83eb51b3",
                        )
                        .unwrap(),
                    ),
                    (
                        "peggy0x44C21afAaF20c270EBbF5914Cfc3b5022173FEB7".to_string(),
                        PriceIdentifier::from_hex(
                            "2d9315a88f3019f8efa88dfe9c0f0843712da0bac814461e27733f6b83eb51b3",
                        )
                        .unwrap(),
                    ),
                ],
                pyth_contract_addr: "inj1z60tg0tekdzcasenhuuwq3htjcd5slmgf7gpez".to_string(),
                price_updater_addr: "".to_string(),
            },
            &[],
            "Contract",
            Some("owner".to_string()), // contract that can execute migrations
        )
        .unwrap();

    app.send_tokens(
        Addr::unchecked("owner"),
        addr.clone(),
        &[coin(CONTRACT_RESERVES_ATOM, "atom")],
    )
    .unwrap();

    app.send_tokens(
        Addr::unchecked("owner"),
        addr.clone(),
        &coins(CONTRACT_RESERVES_ETH, "eth"),
    )
    .unwrap();

    app.execute_contract(
        Addr::unchecked("owner"),
        addr.clone(),
        &ExecuteMsg::UpdatePrice {
            denom: Some("eth".to_string()),
            price: Some(PRICE_ETH),
        },
        &[],
    )
    .unwrap();

    app.execute_contract(
        Addr::unchecked("owner"),
        addr.clone(),
        &ExecuteMsg::UpdatePrice {
            denom: Some("atom".to_string()),
            price: Some(PRICE_ATOM),
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
        &coins(DEPOSIT_AMOUNT_ETH, "eth"),
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

    assert_eq!(user_deposited_balance.balance.u128(), DEPOSIT_AMOUNT_ETH);

    assert_eq!(
        app.wrap()
            .query_balance("user", "eth")
            .unwrap()
            .amount
            .u128(),
        INIT_BALANCE_ETH - DEPOSIT_AMOUNT_ETH
    );

    assert_eq!(
        app.wrap()
            .query_balance(&addr, "eth")
            .unwrap()
            .amount
            .u128(),
        CONTRACT_RESERVES_ETH + DEPOSIT_AMOUNT_ETH
    );

    app.execute_contract(
        Addr::unchecked("user"),
        addr.clone(),
        &ExecuteMsg::Deposit {},
        &coins(DEPOSIT_AMOUNT_ATOM, "atom"),
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

    assert_eq!(user_deposited_balance.balance.u128(), DEPOSIT_AMOUNT_ATOM);

    assert_eq!(
        app.wrap()
            .query_balance("user", "atom")
            .unwrap()
            .amount
            .u128(),
        INIT_BALANCE_ATOM - DEPOSIT_AMOUNT_ATOM
    );

    assert_eq!(
        app.wrap()
            .query_balance(&addr, "atom")
            .unwrap()
            .amount
            .u128(),
        CONTRACT_RESERVES_ATOM + DEPOSIT_AMOUNT_ATOM
    );

    (app, addr)
}

pub fn success_deposit_as_collateral_of_diff_token_with_prices() -> (BasicApp, Addr) {
    let (mut app, addr) = success_deposit_of_diff_token_with_prices();

    app.execute_contract(
        Addr::unchecked("user"),
        addr.clone(),
        &ExecuteMsg::ToggleCollateralSetting {
            denom: "eth".to_string(),
        },
        &[],
    )
    .unwrap();

    app.execute_contract(
        Addr::unchecked("user"),
        addr.clone(),
        &ExecuteMsg::ToggleCollateralSetting {
            denom: "atom".to_string(),
        },
        &[],
    )
    .unwrap();

    app.execute_contract(
        Addr::unchecked("owner"),
        addr.clone(),
        &ExecuteMsg::ToggleCollateralSetting {
            denom: "eth".to_string(),
        },
        &[],
    )
    .unwrap();

    app.execute_contract(
        Addr::unchecked("owner"),
        addr.clone(),
        &ExecuteMsg::ToggleCollateralSetting {
            denom: "atom".to_string(),
        },
        &[],
    )
    .unwrap();

    (app, addr)
}

pub fn success_borrow_setup() -> (BasicApp, Addr) {
    const TOKENS_DECIMALS: u32 = 18;

    const INIT_BALANCE_ETH: u128 = 10_000 * 10u128.pow(TOKENS_DECIMALS); // 10_000 ETH
    const INIT_BALANCE_ATOM: u128 = 10_000 * 10u128.pow(TOKENS_DECIMALS); // 10_000 ATOM
    const INIT_BALANCE_USDT: u128 = 10_000 * 10u128.pow(TOKENS_DECIMALS); // 10_000 USDT

    const INIT_LIQUIDATOR_BALANCE_ETH: u128 = 1_000_000 * 10u128.pow(TOKENS_DECIMALS); // 1M ETH
    const INIT_LIQUIDATOR_BALANCE_ATOM: u128 = 1_000_000 * 10u128.pow(TOKENS_DECIMALS); // 1M ATOM

    const DEPOSIT_AMOUNT_ETH: u128 = 200 * 10u128.pow(TOKENS_DECIMALS);
    const DEPOSIT_AMOUNT_ATOM: u128 = 300 * 10u128.pow(TOKENS_DECIMALS);

    const CONTRACT_RESERVES_ETH: u128 = 1000 * 10u128.pow(TOKENS_DECIMALS);
    const CONTRACT_RESERVES_ATOM: u128 = 1000 * 10u128.pow(TOKENS_DECIMALS);

    const BORROW_AMOUNT_ETH: u128 = 50 * 10u128.pow(TOKENS_DECIMALS);

    const PERCENT_DECIMALS: u32 = 5;
    const LTV_ETH: u128 = 85 * 10u128.pow(PERCENT_DECIMALS); // 85%
    const LIQUIDATION_THRESHOLD_ETH: u128 = 90 * 10u128.pow(PERCENT_DECIMALS); // 90%
    const LTV_ATOM: u128 = 75 * 10u128.pow(PERCENT_DECIMALS); // 75%
    const LIQUIDATION_THRESHOLD_ATOM: u128 = 80 * 10u128.pow(PERCENT_DECIMALS); // 80%

    const INTEREST_RATE_DECIMALS: u32 = 18;
    const MIN_INTEREST_RATE: u128 = 5 * 10u128.pow(INTEREST_RATE_DECIMALS);
    const SAFE_BORROW_MAX_RATE: u128 = 30 * 10u128.pow(INTEREST_RATE_DECIMALS);
    const RATE_GROWTH_FACTOR: u128 = 70 * 10u128.pow(INTEREST_RATE_DECIMALS);

    const OPTIMAL_UTILISATION_RATIO: u128 = 80 * 10u128.pow(PERCENT_DECIMALS);

    const PRICE_DECIMALS: u32 = 8;
    const PRICE_ETH: u128 = 2000 * 10u128.pow(PRICE_DECIMALS);
    const PRICE_ATOM: u128 = 10 * 10u128.pow(PRICE_DECIMALS);

    let mut app = App::new(|router, _, storage| {
        router
            .bank
            .init_balance(
                storage,
                &Addr::unchecked("user"),
                vec![
                    coin(INIT_BALANCE_ETH, "eth"),
                    coin(INIT_BALANCE_ATOM, "atom"),
                    coin(INIT_BALANCE_USDT, "usdt"),
                ],
            )
            .unwrap();

        router
            .bank
            .init_balance(
                storage,
                &Addr::unchecked("owner"),
                vec![
                    coin(CONTRACT_RESERVES_ETH, "eth"),
                    coin(CONTRACT_RESERVES_ATOM, "atom"),
                ],
            )
            .unwrap();

        router
            .bank
            .init_balance(
                storage,
                &Addr::unchecked("liquidator"),
                vec![
                    coin(INIT_LIQUIDATOR_BALANCE_ETH, "eth"),
                    coin(INIT_LIQUIDATOR_BALANCE_ATOM, "atom"),
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
                is_testing: true,
                admin: "owner".to_string(),
                supported_tokens: vec![
                    (
                        "eth".to_string(),
                        "ethereum".to_string(),
                        "ETH".to_string(),
                        None,
                        TOKENS_DECIMALS as u128,
                    ),
                    (
                        "atom".to_string(),
                        "atom".to_string(),
                        "ATOM".to_string(),
                        None,
                        TOKENS_DECIMALS as u128,
                    ),
                ],
                reserve_configuration: vec![
                    ("eth".to_string(), LTV_ETH, LIQUIDATION_THRESHOLD_ETH),
                    ("atom".to_string(), LTV_ATOM, LIQUIDATION_THRESHOLD_ATOM),
                ],
                tokens_interest_rate_model_params: vec![
                    (
                        "eth".to_string(),
                        MIN_INTEREST_RATE,
                        SAFE_BORROW_MAX_RATE,
                        RATE_GROWTH_FACTOR,
                        OPTIMAL_UTILISATION_RATIO,
                    ),
                    (
                        "atom".to_string(),
                        MIN_INTEREST_RATE,
                        SAFE_BORROW_MAX_RATE,
                        RATE_GROWTH_FACTOR,
                        OPTIMAL_UTILISATION_RATIO,
                    ),
                ],
                price_ids: vec![
                    (
                        "inj".to_string(),
                        PriceIdentifier::from_hex(
                            "2d9315a88f3019f8efa88dfe9c0f0843712da0bac814461e27733f6b83eb51b3",
                        )
                        .unwrap(),
                    ),
                    (
                        "peggy0x44C21afAaF20c270EBbF5914Cfc3b5022173FEB7".to_string(),
                        PriceIdentifier::from_hex(
                            "2d9315a88f3019f8efa88dfe9c0f0843712da0bac814461e27733f6b83eb51b3",
                        )
                        .unwrap(),
                    ),
                ],
                pyth_contract_addr: "inj1z60tg0tekdzcasenhuuwq3htjcd5slmgf7gpez".to_string(),
                price_updater_addr: "".to_string(),
            },
            &[],
            "Contract",
            Some("owner".to_string()), // contract that can execute migrations
        )
        .unwrap();

    // funding contract with  reserves
    app.send_tokens(
        Addr::unchecked("owner"),
        addr.clone(),
        &coins(CONTRACT_RESERVES_ETH, "eth"),
    )
    .unwrap();

    app.send_tokens(
        Addr::unchecked("owner"),
        addr.clone(),
        &[coin(CONTRACT_RESERVES_ATOM, "atom")],
    )
    .unwrap();

    app.execute_contract(
        Addr::unchecked("owner"),
        addr.clone(),
        &ExecuteMsg::UpdatePrice {
            denom: Some("eth".to_string()),
            price: Some(PRICE_ETH),
        },
        &[],
    )
    .unwrap();

    app.execute_contract(
        Addr::unchecked("owner"),
        addr.clone(),
        &ExecuteMsg::UpdatePrice {
            denom: Some("atom".to_string()),
            price: Some(PRICE_ATOM),
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

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    app.set_block(BlockInfo {
        height: 0,
        time: Timestamp::from_seconds(now),
        chain_id: "custom_chain_id".to_string(),
    });

    app.execute_contract(
        Addr::unchecked("user"),
        addr.clone(),
        &ExecuteMsg::ToggleCollateralSetting {
            denom: "eth".to_string(),
        },
        &[],
    )
    .unwrap();

    app.execute_contract(
        Addr::unchecked("user"),
        addr.clone(),
        &ExecuteMsg::ToggleCollateralSetting {
            denom: "atom".to_string(),
        },
        &[],
    )
    .unwrap();

    app.execute_contract(
        Addr::unchecked("owner"),
        addr.clone(),
        &ExecuteMsg::ToggleCollateralSetting {
            denom: "eth".to_string(),
        },
        &[],
    )
    .unwrap();

    app.execute_contract(
        Addr::unchecked("owner"),
        addr.clone(),
        &ExecuteMsg::ToggleCollateralSetting {
            denom: "atom".to_string(),
        },
        &[],
    )
    .unwrap();

    app.execute_contract(
        Addr::unchecked("user"),
        addr.clone(),
        &ExecuteMsg::Deposit {},
        &coins(DEPOSIT_AMOUNT_ETH, "eth"),
    )
    .unwrap();

    app.set_block(BlockInfo {
        height: 0,
        time: Timestamp::from_seconds(now + 1000),
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

    assert_eq!(user_deposited_balance.balance.u128(), DEPOSIT_AMOUNT_ETH);

    assert_eq!(
        app.wrap()
            .query_balance("user", "eth")
            .unwrap()
            .amount
            .u128(),
        INIT_BALANCE_ETH - DEPOSIT_AMOUNT_ETH
    );

    assert_eq!(
        app.wrap()
            .query_balance(&addr, "eth")
            .unwrap()
            .amount
            .u128(),
        CONTRACT_RESERVES_ETH + DEPOSIT_AMOUNT_ETH
    );

    app.execute_contract(
        Addr::unchecked("user"),
        addr.clone(),
        &ExecuteMsg::Deposit {},
        &coins(DEPOSIT_AMOUNT_ATOM, "atom"),
    )
    .unwrap();

    app.set_block(BlockInfo {
        height: 0,
        time: Timestamp::from_seconds(now + 2000),
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

    assert_eq!(user_deposited_balance.balance.u128(), DEPOSIT_AMOUNT_ATOM);

    assert_eq!(
        app.wrap()
            .query_balance("user", "atom")
            .unwrap()
            .amount
            .u128(),
        INIT_BALANCE_ATOM - DEPOSIT_AMOUNT_ATOM
    );

    assert_eq!(
        app.wrap()
            .query_balance(&addr, "atom")
            .unwrap()
            .amount
            .u128(),
        CONTRACT_RESERVES_ATOM + DEPOSIT_AMOUNT_ATOM
    );

    app.set_block(BlockInfo {
        height: 542,
        time: Timestamp::from_seconds(now + 10000),
        chain_id: "custom_chain_id".to_string(),
    });

    app.execute_contract(
        Addr::unchecked("user"),
        addr.clone(),
        &ExecuteMsg::Borrow {
            denom: "eth".to_string(),
            amount: Uint128::from(BORROW_AMOUNT_ETH),
        },
        &[],
    )
    .unwrap();

    (app, addr)
}

pub fn success_native_and_cw20_setup() -> (BasicApp, Addr, Addr) {
    const TOKENS_DECIMALS: u32 = 18;

    const INIT_USER_BALANCE: u128 = 100000 * 10u128.pow(TOKENS_DECIMALS);
    const INIT_LIQUIDATOR_BALANCE_ETH: u128 = 1_000_000 * 10u128.pow(TOKENS_DECIMALS); // 1M ETH

    const CONTRACT_RESERVES: u128 = 10000 * 10u128.pow(TOKENS_DECIMALS);
    const FIRST_DEPOSIT_AMOUNT_ETH: u128 = 200 * 10u128.pow(TOKENS_DECIMALS);
    const SECOND_DEPOSIT_AMOUNT_ETH: u128 = 300 * 10u128.pow(TOKENS_DECIMALS);

    const PERCENT_DECIMALS: u32 = 5;
    const LTV_ETH: u128 = 85 * 10u128.pow(PERCENT_DECIMALS); // 85%
    const LIQUIDATION_THRESHOLD_ETH: u128 = 90 * 10u128.pow(PERCENT_DECIMALS); // 90%
    const LTV_ATOM: u128 = 75 * 10u128.pow(PERCENT_DECIMALS); // 75%
    const LIQUIDATION_THRESHOLD_ATOM: u128 = 80 * 10u128.pow(PERCENT_DECIMALS); // 80%

    const INTEREST_RATE_DECIMALS: u32 = 18;
    const MIN_INTEREST_RATE: u128 = 5 * 10u128.pow(INTEREST_RATE_DECIMALS);
    const SAFE_BORROW_MAX_RATE: u128 = 30 * 10u128.pow(INTEREST_RATE_DECIMALS);
    const RATE_GROWTH_FACTOR: u128 = 70 * 10u128.pow(INTEREST_RATE_DECIMALS);

    const OPTIMAL_UTILISATION_RATIO: u128 = 80 * 10u128.pow(PERCENT_DECIMALS);

    let mut app = App::new(|router, _, storage| {
        router
            .bank
            .init_balance(
                storage,
                &Addr::unchecked("cw20-user"),
                coins(INIT_USER_BALANCE, "eth"),
            )
            .unwrap();

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
            .unwrap();

        router
            .bank
            .init_balance(
                storage,
                &Addr::unchecked("liquidator"),
                coins(INIT_LIQUIDATOR_BALANCE_ETH, "eth"),
            )
            .unwrap();
    });

    let code = ContractWrapper::new(execute, instantiate, query);
    let code_id = app.store_code(Box::new(code));

    let lending_addr = app
        .instantiate_contract(
            code_id,
            Addr::unchecked("owner"),
            &InstantiateMsg {
                is_testing: true,
                price_ids: vec![
                    (
                        "inj".to_string(),
                        PriceIdentifier::from_hex(
                            "2d9315a88f3019f8efa88dfe9c0f0843712da0bac814461e27733f6b83eb51b3",
                        )
                        .unwrap(),
                    ),
                    (
                        "ilend-denom".to_string(),
                        PriceIdentifier::from_hex(
                            "2d9315a88f3019f8efa88dfe9c0f0843712da0bac814461e27733f6b83eb51b3",
                        )
                        .unwrap(),
                    ),
                    (
                        "peggy0x44C21afAaF20c270EBbF5914Cfc3b5022173FEB7".to_string(),
                        PriceIdentifier::from_hex(
                            "2d9315a88f3019f8efa88dfe9c0f0843712da0bac814461e27733f6b83eb51b3",
                        )
                        .unwrap(),
                    ),
                ],
                pyth_contract_addr: "inj1z60tg0tekdzcasenhuuwq3htjcd5slmgf7gpez".to_string(),
                admin: "owner".to_string(),
                supported_tokens: vec![
                    (
                        "eth".to_string(),
                        "ethereum".to_string(),
                        "ETH".to_string(),
                        None,
                        TOKENS_DECIMALS as u128,
                    ),
                    (
                        "atom".to_string(),
                        "atom".to_string(),
                        "ATOM".to_string(),
                        None,
                        TOKENS_DECIMALS as u128,
                    ),
                ],
                reserve_configuration: vec![
                    ("eth".to_string(), LTV_ETH, LIQUIDATION_THRESHOLD_ETH),
                    ("atom".to_string(), LTV_ATOM, LIQUIDATION_THRESHOLD_ATOM),
                ],
                tokens_interest_rate_model_params: vec![
                    (
                        "eth".to_string(),
                        MIN_INTEREST_RATE,
                        SAFE_BORROW_MAX_RATE,
                        RATE_GROWTH_FACTOR,
                        OPTIMAL_UTILISATION_RATIO,
                    ),
                    (
                        "atom".to_string(),
                        MIN_INTEREST_RATE,
                        SAFE_BORROW_MAX_RATE,
                        RATE_GROWTH_FACTOR,
                        OPTIMAL_UTILISATION_RATIO,
                    ),
                ],
                price_updater_addr: "".to_string(),
            },
            &[],
            "Contract",
            Some("owner".to_string()), // contract that can execute migrations
        )
        .unwrap();

    let cw20_token_code =
        ContractWrapper::new_with_empty(execute_cw20, instantiate_cw20, query_cw20);
    let cw20_token_code_id = app.store_code(Box::new(cw20_token_code));

    let cw20_init_msg = InstantiateMsgCW20 {
        name: "Ilend Test Tokens".to_string(),
        symbol: "ILEND".to_string(),
        decimals: 6,
        initial_balances: vec![
            Cw20Coin {
                address: String::from(Addr::unchecked("cw20-user")),
                amount: Uint128::from(10000000000000000u128),
            },
            Cw20Coin {
                address: String::from(&Addr::unchecked("owner-token")),
                amount: Uint128::from(200000000000000000u128),
            },
            Cw20Coin {
                address: String::from(&Addr::unchecked("liquidator")),
                amount: Uint128::from(20000000000u128),
            },
            Cw20Coin {
                address: lending_addr.to_string(),
                amount: Uint128::from(1000000000000u128),
            },
        ],
        mint: Some(MinterResponse {
            minter: "owner-token".to_string(),
            cap: Some(Uint128::from(20000000000000000000000000000000u128)),
        }),
        marketing: None,
    };

    let cw20_token_addr = app
        .instantiate_contract(
            cw20_token_code_id,
            Addr::unchecked("owner-token"),
            &cw20_init_msg,
            &[],
            "CW20 token contract",
            Some("owner-token".to_string()),
        )
        .unwrap();

    let owner_token_initial_amount: BalanceResponse = app
        .wrap()
        .query_wasm_smart(
            cw20_token_addr.clone(),
            &Cw20QueryMsg::Balance {
                address: "owner-token".to_string(),
            },
        )
        .unwrap();

    let cw20_user_token_initial_amount: BalanceResponse = app
        .wrap()
        .query_wasm_smart(
            cw20_token_addr.clone(),
            &Cw20QueryMsg::Balance {
                address: "cw20-user".to_string(),
            },
        )
        .unwrap();

    let liquidator_token_initial_amount: BalanceResponse = app
        .wrap()
        .query_wasm_smart(
            cw20_token_addr.clone(),
            &Cw20QueryMsg::Balance {
                address: "liquidator".to_string(),
            },
        )
        .unwrap();

    assert_eq!(
        owner_token_initial_amount.balance.u128(),
        200000000000000000,
        "Has to be equal to the initially obtained amount"
    );
    assert_eq!(
        cw20_user_token_initial_amount.balance.u128(),
        10000000000000000,
        "Has to be equal to the initially obtained amount"
    );
    assert_eq!(
        liquidator_token_initial_amount.balance.u128(),
        20000000000,
        "Has to be equal to the initially obtained amount"
    );

    app.send_tokens(
        Addr::unchecked("owner"),
        lending_addr.clone(),
        &[coin(CONTRACT_RESERVES, "eth")],
    )
    .unwrap();

    app.execute_contract(
        Addr::unchecked("user"),
        lending_addr.clone(),
        &ExecuteMsg::Deposit {},
        &coins(FIRST_DEPOSIT_AMOUNT_ETH, "eth"),
    )
    .unwrap();

    let user_deposited_balance: GetBalanceResponse = app
        .wrap()
        .query_wasm_smart(
            lending_addr.clone(),
            &QueryMsg::GetDeposit {
                address: "user".to_string(),
                denom: "eth".to_string(),
            },
        )
        .unwrap();
    assert_eq!(
        user_deposited_balance.balance.u128(),
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
            .query_balance(&lending_addr, "eth")
            .unwrap()
            .amount
            .u128(),
        CONTRACT_RESERVES + FIRST_DEPOSIT_AMOUNT_ETH
    );

    app.execute_contract(
        Addr::unchecked("user"),
        lending_addr.clone(),
        &ExecuteMsg::Deposit {},
        &coins(SECOND_DEPOSIT_AMOUNT_ETH, "eth"),
    )
    .unwrap();

    let user_deposited_balance: GetBalanceResponse = app
        .wrap()
        .query_wasm_smart(
            lending_addr.clone(),
            &QueryMsg::GetDeposit {
                address: "user".to_string(),
                denom: "eth".to_string(),
            },
        )
        .unwrap();

    assert_eq!(
        user_deposited_balance.balance.u128(),
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
            .query_balance(&lending_addr, "eth")
            .unwrap()
            .amount
            .u128(),
        CONTRACT_RESERVES + FIRST_DEPOSIT_AMOUNT_ETH + SECOND_DEPOSIT_AMOUNT_ETH
    );

    (app, lending_addr, cw20_token_addr)
}
