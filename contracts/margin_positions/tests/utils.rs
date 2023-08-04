use cosmwasm_std::{coin, coins, Addr, Empty};
use cw_multi_test::{custom_app, BasicApp, ContractWrapper, Executor};
use std::vec;

use collateral_vault::msg::{
    ExecuteMsg as ExecuteMsgCollateralVault, InstantiateMsg as InstantiateMsgCollateralVault,
    QueryMsg as QueryMsgCollateralVault,
};

use margin_positions::msg::{
    ExecuteMsg as ExecuteMsgMarginPositions, InstantiateMsg as InstantiateMsgMarginPositions,
    QueryMsg as QueryMsgMarginPositions,
};

use collateral_vault::{
    execute as execute_collateral_vault, instantiate as instantiate_collateral_vault,
    query as query_collateral_vault,
};
use margin_positions::{
    execute as execute_margin_positions, instantiate as instantiate_margin_positions,
    query as query_margin_positions,
};

use cosmwasm_std::Uint128;

use pyth_sdk_cw::PriceIdentifier;

pub fn success_setup_collateral_vault_and_margin_contract() -> (BasicApp<CustomMsg>, Addr, Addr) {
    const TOKENS_DECIMALS: u32 = 18;

    const INIT_USER_BALANCE: u128 = 1000 * 10u128.pow(TOKENS_DECIMALS);

    const RESERVE_AMOUNT: u128 = 1000 * 10u128.pow(TOKENS_DECIMALS);

    const CONTRACT_RESERVES: u128 = 1000000 * 10u128.pow(TOKENS_DECIMALS);

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
                    coin(INIT_USER_BALANCE, "eth"),
                    coin(INIT_USER_BALANCE, "atom"),
                ],
            )
            .unwrap();

        router
            .bank
            .init_balance(
                storage,
                &Addr::unchecked("margin_positions"),
                vec![
                    coin(CONTRACT_RESERVES, "eth"),
                    coin(CONTRACT_RESERVES, "atom"),
                ],
            )
            .unwrap();

        router
            .bank
            .init_balance(
                storage,
                &Addr::unchecked("collateral_vault"),
                vec![
                    coin(10 * RESERVE_AMOUNT, "eth"),
                    coin(10 * RESERVE_AMOUNT, "atom"),
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
        &ExecuteMsgCollateralVault::Fund {},
        &[coin(RESERVE_AMOUNT, "eth")],
    )
    .unwrap();

    app.execute_contract(
        Addr::unchecked("collateral_vault"),
        collateral_contract_addr.clone(),
        &ExecuteMsgCollateralVault::Fund {},
        &[coin(RESERVE_AMOUNT, "atom")],
    )
    .unwrap();

    let code = ContractWrapper::new_with_empty(
        execute_margin_positions,
        instantiate_margin_positions,
        query_margin_positions,
    );
    let code_id = app.store_code(Box::new(code));

    let margin_positions_addr = app
        .instantiate_contract(
            code_id,
            Addr::unchecked("margin_positions"),
            &InstantiateMsgMarginPositions {
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
                        TOKENS_DECIMALS as u128,
                    ),
                    (
                        "atom".to_string(),
                        "atom".to_string(),
                        "ATOM".to_string(),
                        TOKENS_DECIMALS as u128,
                    ),
                ],

                price_updater_contract_addr: "".to_string(),
                collateral_vault_contract: collateral_contract_addr.to_string(),
                lending_contract: "whatever".to_string(),
            },
            &[],
            "Margin Positions Contract",
            Some("margin_positions".to_string()), // contract that can execute migrations
        )
        .unwrap();

    app.execute_contract(
        Addr::unchecked("owner"),
        margin_positions_addr.clone(),
        &ExecuteMsgMarginPositions::UpdatePrice {
            denom: Some("eth".to_string()),
            price: Some(PRICE_ETH),
        },
        &[],
    )
    .unwrap();

    app.execute_contract(
        Addr::unchecked("owner"),
        margin_positions_addr.clone(),
        &ExecuteMsgMarginPositions::UpdatePrice {
            denom: Some("atom".to_string()),
            price: Some(PRICE_ATOM),
        },
        &[],
    )
    .unwrap();

    let get_price_eth: Uint128 = app
        .wrap()
        .query_wasm_smart(
            margin_positions_addr.clone(),
            &QueryMsgMarginPositions::GetPrice {
                denom: "eth".to_string(),
            },
        )
        .unwrap();

    let get_price_atom: Uint128 = app
        .wrap()
        .query_wasm_smart(
            margin_positions_addr.clone(),
            &QueryMsgMarginPositions::GetPrice {
                denom: "atom".to_string(),
            },
        )
        .unwrap();

    assert_eq!(get_price_atom.u128(), 1000000000); // 10$
    assert_eq!(get_price_eth.u128(), 200000000000); // 2000$

    app.execute_contract(
        Addr::unchecked("collateral_vault"),
        collateral_contract_addr.clone(),
        &ExecuteMsgCollateralVault::SetMarginContract {
            contract: margin_positions_addr.to_string(),
        },
        &[],
    )
    .unwrap();

    let margin_contract: String = app
        .wrap()
        .query_wasm_smart(
            collateral_contract_addr.clone(),
            &QueryMsgCollateralVault::GetMarginContract {},
        )
        .unwrap();

    assert_eq!(margin_contract, margin_positions_addr.to_string());

    (app, margin_positions_addr, collateral_contract_addr)
}

pub fn success_collateral_margin_setup_with_deposit() -> (BasicApp<CustomMsg>, Addr, Addr) {
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
        &ExecuteMsgMarginPositions::Deposit {},
        &coins(FIRST_DEPOSIT_AMOUNT_ETH, "eth"),
    )
    .unwrap();

    let user_deposited_balance: Uint128 = app
        .wrap()
        .query_wasm_smart(
            margin_positions_addr.clone(),
            &QueryMsgMarginPositions::GetDeposit {
                address: "user".to_string(),
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
        &ExecuteMsgMarginPositions::Deposit {},
        &coins(SECOND_DEPOSIT_AMOUNT_ETH, "eth"),
    )
    .unwrap();

    let user_deposited_balance: Uint128 = app
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

    (app, margin_positions_addr, collateral_contract_addr)
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
    // Deposit / Redeem functionality
    Deposit {},
    Redeem {
        denom: String,
        amount: Uint128,
    },
    SetLendingContract {
        contract: String,
    },
    SetMarginContract {
        contract: String,
    },
    Fund {},
}
