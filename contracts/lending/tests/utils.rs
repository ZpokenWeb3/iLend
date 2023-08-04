use cosmwasm_std::{coin, coins, Addr, BlockInfo, Empty, Timestamp};
use cw_multi_test::{custom_app, BasicApp, ContractWrapper, Executor};
use std::vec;

use collateral_vault::msg::{
    InstantiateMsg as InstantiateMsgCollateralVault,
    QueryMsg as QueryMsgCollateralVault,
    // ExecuteMsg as ExecuteMsgCollateralVault,
};
use collateral_vault::{
    execute as execute_collateral_vault, instantiate as instantiate_collateral_vault,
    query as query_collateral_vault,
};
use cosmwasm_std::Uint128;
use lending::msg::{
    ExecuteCollateralVault, ExecuteMsg, GetBalanceResponse, InstantiateMsg, QueryMsg,
};
use lending::{execute, instantiate, query};
use pyth_sdk_cw::PriceIdentifier;
use std::time::{SystemTime, UNIX_EPOCH};

pub fn success_deposit_of_one_token_setup() -> (BasicApp<CustomMsg>, Addr, Addr) {
    const TOKENS_DECIMALS: u32 = 18;

    const INIT_USER_BALANCE: u128 = 1000 * 10u128.pow(TOKENS_DECIMALS);
    const INIT_LIQUIDATOR_BALANCE_ETH: u128 = 1_000_000 * 10u128.pow(TOKENS_DECIMALS); // 1M ETH

    const RESERVE_AMOUNT: u128 = 1000 * 10u128.pow(TOKENS_DECIMALS);

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

    let mut app = custom_app::<CustomMsg, Empty, _>(|router, _, storage| {
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

        router
            .bank
            .init_balance(
                storage,
                &Addr::unchecked("collateral_vault"),
                coins(10 * RESERVE_AMOUNT, "eth"),
            )
            .unwrap();
    });

    let code_collateral_vault = ContractWrapper::new_with_empty(
        execute_collateral_vault,
        instantiate_collateral_vault,
        query_collateral_vault,
    );
    let code_id_collateral_vault = app.store_code(Box::new(code_collateral_vault));

    let collateral_contract_addr = app
        .instantiate_contract(
            code_id_collateral_vault,
            Addr::unchecked("collateral_vault"),
            &InstantiateMsgCollateralVault {
                lending_contract: "whatever".to_string(),
                margin_contract: "whatever".to_string(),
                admin: "collateral_vault".to_string(),
            },
            &[],
            "Collateral Vault Contract",
            Some("collateral_vault".to_string()), // contract that can execute migrations
        )
        .unwrap();

    app.execute_contract(
        Addr::unchecked("collateral_vault"),
        collateral_contract_addr.clone(),
        &ExecuteCollateralVault::Fund {},
        &[coin(RESERVE_AMOUNT, "eth")],
    )
    .unwrap();

    let code = ContractWrapper::new_with_empty(execute, instantiate, query);
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
                        "peggy0x44C21afAaF20c270EBbF5914Cfc3b5022173FEB7".to_string(),
                        PriceIdentifier::from_hex(
                            "2d9315a88f3019f8efa88dfe9c0f0843712da0bac814461e27733f6b83eb51b3",
                        )
                        .unwrap(),
                    ),
                ],
                pyth_contract_addr: "inj1z60tg0tekdzcasenhuuwq3htjcd5slmgf7gpez".to_string(),
                admin: "owner".to_string(),
                liquidator: "liquidator".to_string(),
                supported_tokens: vec![
                    (
                        "eth".to_string(),
                        "ethereum".to_string(),
                        "ETH".to_string(),
                        TOKENS_DECIMALS as u128,
                    ),
                    (
                        "atom".to_string(),
                        "atom".to_string(),
                        "ATOM".to_string(),
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
                price_updater_contract_addr: "".to_string(),
                collateral_vault_contract: collateral_contract_addr.to_string(),
            },
            &[],
            "Contract",
            Some("owner".to_string()), // contract that can execute migrations
        )
        .unwrap();

    app.execute_contract(
        Addr::unchecked("collateral_vault"),
        collateral_contract_addr.clone(),
        &ExecuteCollateralVault::SetLendingContract {
            contract: lending_addr.to_string(),
        },
        &[],
    )
    .unwrap();

    let lending_contract: String = app
        .wrap()
        .query_wasm_smart(
            collateral_contract_addr.clone(),
            &QueryMsgCollateralVault::GetLendingContract {},
        )
        .unwrap();

    assert_eq!(lending_contract, lending_addr.to_string());

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
            .query_balance(collateral_contract_addr.clone(), "eth")
            .unwrap()
            .amount
            .u128(),
        RESERVE_AMOUNT + FIRST_DEPOSIT_AMOUNT_ETH
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
            .query_balance(collateral_contract_addr.clone(), "eth")
            .unwrap()
            .amount
            .u128(),
        RESERVE_AMOUNT + FIRST_DEPOSIT_AMOUNT_ETH + SECOND_DEPOSIT_AMOUNT_ETH
    );

    (app, lending_addr, collateral_contract_addr)
}

pub fn success_deposit_of_diff_token_with_prices() -> (BasicApp<CustomMsg>, Addr, Addr) {
    const TOKENS_DECIMALS: u32 = 18;

    const INIT_BALANCE_ETH: u128 = 1000 * 10u128.pow(TOKENS_DECIMALS);
    const INIT_BALANCE_ATOM: u128 = 1_000_000 * 10u128.pow(TOKENS_DECIMALS); // 1M ATOM

    const INIT_BALANCE_COLLATERAL_VAULT_ETH: u128 = 10000 * 10u128.pow(TOKENS_DECIMALS);
    const INIT_BALANCE_COLLATERAL_VAULT_ATOM: u128 = 10000 * 10u128.pow(TOKENS_DECIMALS);

    const RESERVE_AMOUNT: u128 = 1000 * 10u128.pow(TOKENS_DECIMALS);

    const INIT_LIQUIDATOR_BALANCE_ETH: u128 = 1_000_000 * 10u128.pow(TOKENS_DECIMALS); // 1M ETH
    const INIT_LIQUIDATOR_BALANCE_ATOM: u128 = 1_000_000 * 10u128.pow(TOKENS_DECIMALS); // 1M ATOM

    const DEPOSIT_AMOUNT_ETH: u128 = 200 * 10u128.pow(TOKENS_DECIMALS);
    const DEPOSIT_AMOUNT_ATOM: u128 = 300 * 10u128.pow(TOKENS_DECIMALS);

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

    let mut app = custom_app::<CustomMsg, Empty, _>(|router, _, storage| {
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

        router
            .bank
            .init_balance(
                storage,
                &Addr::unchecked("collateral_vault"),
                vec![
                    coin(INIT_BALANCE_COLLATERAL_VAULT_ETH, "eth"),
                    coin(INIT_BALANCE_COLLATERAL_VAULT_ATOM, "atom"),
                ],
            )
            .unwrap();
    });

    let code_collateral_vault = ContractWrapper::new_with_empty(
        execute_collateral_vault,
        instantiate_collateral_vault,
        query_collateral_vault,
    );
    let code_id_collateral_vault = app.store_code(Box::new(code_collateral_vault));

    let collateral_contract_addr = app
        .instantiate_contract(
            code_id_collateral_vault,
            Addr::unchecked("collateral_vault"),
            &InstantiateMsgCollateralVault {
                lending_contract: "owner".to_string(),
                margin_contract: "whatever".to_string(),
                admin: "collateral_vault".to_string(),
            },
            &[],
            "Collateral Vault Contract",
            Some("collateral_vault".to_string()), // contract that can execute migrations
        )
        .unwrap();

    app.execute_contract(
        Addr::unchecked("collateral_vault"),
        collateral_contract_addr.clone(),
        &ExecuteCollateralVault::Fund {},
        &[coin(RESERVE_AMOUNT, "eth")],
    )
    .unwrap();

    app.execute_contract(
        Addr::unchecked("collateral_vault"),
        collateral_contract_addr.clone(),
        &ExecuteCollateralVault::Fund {},
        &[coin(RESERVE_AMOUNT, "atom")],
    )
    .unwrap();

    let code = ContractWrapper::new_with_empty(execute, instantiate, query);
    let code_id = app.store_code(Box::new(code));

    let lending_addr = app
        .instantiate_contract(
            code_id,
            Addr::unchecked("owner"),
            &InstantiateMsg {
                is_testing: true,
                admin: "owner".to_string(),
                liquidator: "liquidator".to_string(),
                supported_tokens: vec![
                    (
                        "eth".to_string(),
                        "ethereum".to_string(),
                        "ETH".to_string(),
                        TOKENS_DECIMALS as u128,
                    ),
                    (
                        "atom".to_string(),
                        "atom".to_string(),
                        "ATOM".to_string(),
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
                price_updater_contract_addr: "".to_string(),
                collateral_vault_contract: collateral_contract_addr.to_string(),
            },
            &[],
            "Contract",
            Some("owner".to_string()), // contract that can execute migrations
        )
        .unwrap();

    app.execute_contract(
        Addr::unchecked("collateral_vault"),
        collateral_contract_addr.clone(),
        &ExecuteCollateralVault::SetLendingContract {
            contract: lending_addr.to_string(),
        },
        &[],
    )
    .unwrap();

    let lending_contract: String = app
        .wrap()
        .query_wasm_smart(
            collateral_contract_addr.clone(),
            &QueryMsgCollateralVault::GetLendingContract {},
        )
        .unwrap();

    assert_eq!(lending_contract, lending_addr.to_string());

    app.execute_contract(
        Addr::unchecked("owner"),
        lending_addr.clone(),
        &ExecuteMsg::UpdatePrice {
            denom: Some("eth".to_string()),
            price: Some(PRICE_ETH),
        },
        &[],
    )
    .unwrap();

    app.execute_contract(
        Addr::unchecked("owner"),
        lending_addr.clone(),
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
            lending_addr.clone(),
            &QueryMsg::GetPrice {
                denom: "eth".to_string(),
            },
        )
        .unwrap();

    let get_price_atom: Uint128 = app
        .wrap()
        .query_wasm_smart(
            lending_addr.clone(),
            &QueryMsg::GetPrice {
                denom: "atom".to_string(),
            },
        )
        .unwrap();

    assert_eq!(get_price_atom.u128(), 1000000000); // 10$
    assert_eq!(get_price_eth.u128(), 200000000000); // 2000$

    app.execute_contract(
        Addr::unchecked("user"),
        lending_addr.clone(),
        &ExecuteMsg::Deposit {},
        &coins(DEPOSIT_AMOUNT_ETH, "eth"),
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
            .query_balance(collateral_contract_addr.clone(), "eth")
            .unwrap()
            .amount
            .u128(),
        RESERVE_AMOUNT + DEPOSIT_AMOUNT_ETH
    );

    app.execute_contract(
        Addr::unchecked("user"),
        lending_addr.clone(),
        &ExecuteMsg::Deposit {},
        &coins(DEPOSIT_AMOUNT_ATOM, "atom"),
    )
    .unwrap();

    let user_deposited_balance: GetBalanceResponse = app
        .wrap()
        .query_wasm_smart(
            lending_addr.clone(),
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
            .query_balance(collateral_contract_addr.clone(), "atom")
            .unwrap()
            .amount
            .u128(),
        RESERVE_AMOUNT + DEPOSIT_AMOUNT_ATOM
    );

    (app, lending_addr, collateral_contract_addr)
}

pub fn success_deposit_as_collateral_of_diff_token_with_prices() -> (BasicApp<CustomMsg>, Addr, Addr)
{
    let (mut app, addr, collateral_contract_addr) = success_deposit_of_diff_token_with_prices();

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

    (app, addr, collateral_contract_addr)
}

pub fn success_borrow_setup() -> (BasicApp<CustomMsg>, Addr, Addr) {
    const TOKENS_DECIMALS: u32 = 18;

    const INIT_BALANCE_ETH: u128 = 10_000 * 10u128.pow(TOKENS_DECIMALS); // 10_000 ETH
    const INIT_BALANCE_ATOM: u128 = 10_000 * 10u128.pow(TOKENS_DECIMALS); // 10_000 ATOM

    const INIT_BALANCE_USDT: u128 = 10_000 * 10u128.pow(TOKENS_DECIMALS); // 10_000 USDT

    const INIT_LIQUIDATOR_BALANCE_ETH: u128 = 1_000_000 * 10u128.pow(TOKENS_DECIMALS); // 1M ETH
    const INIT_LIQUIDATOR_BALANCE_ATOM: u128 = 1_000_000 * 10u128.pow(TOKENS_DECIMALS); // 1M ATOM

    const DEPOSIT_AMOUNT_ETH: u128 = 200 * 10u128.pow(TOKENS_DECIMALS);
    const DEPOSIT_AMOUNT_ATOM: u128 = 300 * 10u128.pow(TOKENS_DECIMALS);

    const INIT_BALANCE_COLLATERAL_VAULT_ETH: u128 = 10000 * 10u128.pow(TOKENS_DECIMALS);
    const INIT_BALANCE_COLLATERAL_VAULT_ATOM: u128 = 10000 * 10u128.pow(TOKENS_DECIMALS);

    const RESERVE_AMOUNT: u128 = 1000 * 10u128.pow(TOKENS_DECIMALS);

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

    let mut app = custom_app::<CustomMsg, Empty, _>(|router, _, storage| {
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
                    coin(INIT_BALANCE_COLLATERAL_VAULT_ETH, "eth"),
                    coin(INIT_BALANCE_COLLATERAL_VAULT_ATOM, "atom"),
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

        router
            .bank
            .init_balance(
                storage,
                &Addr::unchecked("collateral_vault"),
                vec![
                    coin(INIT_LIQUIDATOR_BALANCE_ETH, "eth"),
                    coin(INIT_LIQUIDATOR_BALANCE_ATOM, "atom"),
                ],
            )
            .unwrap();
    });

    let code_collateral_vault = ContractWrapper::new_with_empty(
        execute_collateral_vault,
        instantiate_collateral_vault,
        query_collateral_vault,
    );

    let code_id_collateral_vault = app.store_code(Box::new(code_collateral_vault));

    let collateral_contract_addr = app
        .instantiate_contract(
            code_id_collateral_vault,
            Addr::unchecked("collateral_vault"),
            &InstantiateMsgCollateralVault {
                lending_contract: "whatever".to_string(),
                margin_contract: "whatever".to_string(),
                admin: "collateral_vault".to_string(),
            },
            &[],
            "Collateral Vault Contract",
            Some("collateral_vault".to_string()), // contract that can execute migrations
        )
        .unwrap();

    app.execute_contract(
        Addr::unchecked("collateral_vault"),
        collateral_contract_addr.clone(),
        &ExecuteCollateralVault::Fund {},
        &[coin(RESERVE_AMOUNT, "eth")],
    )
    .unwrap();

    app.execute_contract(
        Addr::unchecked("collateral_vault"),
        collateral_contract_addr.clone(),
        &ExecuteCollateralVault::Fund {},
        &[coin(RESERVE_AMOUNT, "atom")],
    )
    .unwrap();

    let code = ContractWrapper::new_with_empty(execute, instantiate, query);
    let code_id = app.store_code(Box::new(code));

    let lending_addr = app
        .instantiate_contract(
            code_id,
            Addr::unchecked("owner"),
            &InstantiateMsg {
                is_testing: true,
                admin: "owner".to_string(),
                liquidator: "liquidator".to_string(),
                supported_tokens: vec![
                    (
                        "eth".to_string(),
                        "ethereum".to_string(),
                        "ETH".to_string(),
                        TOKENS_DECIMALS as u128,
                    ),
                    (
                        "atom".to_string(),
                        "atom".to_string(),
                        "ATOM".to_string(),
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
                price_updater_contract_addr: "".to_string(),
                collateral_vault_contract: collateral_contract_addr.to_string(),
            },
            &[],
            "Contract",
            Some("owner".to_string()), // contract that can execute migrations
        )
        .unwrap();

    app.execute_contract(
        Addr::unchecked("collateral_vault"),
        collateral_contract_addr.clone(),
        &ExecuteCollateralVault::SetLendingContract {
            contract: lending_addr.to_string(),
        },
        &[],
    )
    .unwrap();

    let lending_contract: String = app
        .wrap()
        .query_wasm_smart(
            collateral_contract_addr.clone(),
            &QueryMsgCollateralVault::GetLendingContract {},
        )
        .unwrap();

    assert_eq!(lending_contract, lending_addr.to_string());

    app.execute_contract(
        Addr::unchecked("owner"),
        lending_addr.clone(),
        &ExecuteMsg::UpdatePrice {
            denom: Some("eth".to_string()),
            price: Some(PRICE_ETH),
        },
        &[],
    )
    .unwrap();

    app.execute_contract(
        Addr::unchecked("owner"),
        lending_addr.clone(),
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
            lending_addr.clone(),
            &QueryMsg::GetPrice {
                denom: "eth".to_string(),
            },
        )
        .unwrap();

    let get_price_atom: Uint128 = app
        .wrap()
        .query_wasm_smart(
            lending_addr.clone(),
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
        lending_addr.clone(),
        &ExecuteMsg::ToggleCollateralSetting {
            denom: "eth".to_string(),
        },
        &[],
    )
    .unwrap();

    app.execute_contract(
        Addr::unchecked("user"),
        lending_addr.clone(),
        &ExecuteMsg::ToggleCollateralSetting {
            denom: "atom".to_string(),
        },
        &[],
    )
    .unwrap();

    app.execute_contract(
        Addr::unchecked("owner"),
        lending_addr.clone(),
        &ExecuteMsg::ToggleCollateralSetting {
            denom: "eth".to_string(),
        },
        &[],
    )
    .unwrap();

    app.execute_contract(
        Addr::unchecked("owner"),
        lending_addr.clone(),
        &ExecuteMsg::ToggleCollateralSetting {
            denom: "atom".to_string(),
        },
        &[],
    )
    .unwrap();

    app.execute_contract(
        Addr::unchecked("user"),
        lending_addr.clone(),
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
            lending_addr.clone(),
            &QueryMsg::GetAvailableToRedeem {
                address: "user".to_string(),
                denom: "eth".to_string(),
            },
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
            .query_balance(&collateral_contract_addr, "eth")
            .unwrap()
            .amount
            .u128(),
        RESERVE_AMOUNT + DEPOSIT_AMOUNT_ETH
    );

    app.execute_contract(
        Addr::unchecked("user"),
        lending_addr.clone(),
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
            lending_addr.clone(),
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
            .query_balance(&collateral_contract_addr, "atom")
            .unwrap()
            .amount
            .u128(),
        RESERVE_AMOUNT + DEPOSIT_AMOUNT_ATOM
    );

    app.set_block(BlockInfo {
        height: 542,
        time: Timestamp::from_seconds(now + 10000),
        chain_id: "custom_chain_id".to_string(),
    });

    app.execute_contract(
        Addr::unchecked("user"),
        lending_addr.clone(),
        &ExecuteMsg::Borrow {
            denom: "eth".to_string(),
            amount: Uint128::from(BORROW_AMOUNT_ETH),
        },
        &[],
    )
    .unwrap();

    (app, lending_addr, collateral_contract_addr)
}

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename = "snake_case")]
pub enum CustomMsg {
    UpdatePrice {
        denom: Option<String>,
        price: Option<u128>,
    },
    SetReserveConfiguration {
        denom: String,
        loan_to_value_ratio: u128,
        liquidation_threshold: u128,
    },
    SetTokenInterestRateModelParams {
        denom: String,
        min_interest_rate: u128,
        safe_borrow_max_rate: u128,
        rate_growth_factor: u128,
        optimal_utilisation_ratio: u128,
    },
    AddMarkets {
        denom: String,
        name: String,
        symbol: String,
        decimals: u128,
        loan_to_value_ratio: u128,
        liquidation_threshold: u128,
        min_interest_rate: u128,
        safe_borrow_max_rate: u128,
        rate_growth_factor: u128,
        optimal_utilisation_ratio: u128,
    },

    // Deposit / Redeem functionality
    Deposit {},
    Redeem {
        denom: String,
        amount: Uint128,
    },

    // Borrow / Repay functionality
    Borrow {
        denom: String,
        amount: Uint128,
    },
    Repay {},
    ToggleCollateralSetting {
        denom: String,
    },
    Liquidation {
        user: String,
    },
    SetLendingContract {
        contract: String,
    },
    SetMarginContract {
        contract: String,
    },
    RedeemFromVaultContract {
        denom: String,
        amount: Uint128,
        user: String,
    },
    BorrowFromVaultContract {
        denom: String,
        amount: Uint128,
        user: String,
    },
    Fund {},
}
