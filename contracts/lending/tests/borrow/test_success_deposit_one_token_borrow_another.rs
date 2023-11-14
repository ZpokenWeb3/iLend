#[cfg(test)]
mod tests {
    use crate::utils::CustomMsg;
    use collateral_vault::msg::{
        InstantiateMsg as InstantiateMsgCollateralVault, QueryMsg as QueryMsgCollateralVault,
    };
    use collateral_vault::{
        execute as execute_collateral_vault, instantiate as instantiate_collateral_vault,
        query as query_collateral_vault,
    };
    use cosmwasm_std::{coin, coins, Addr, Empty, Uint128};
    use cw_multi_test::{custom_app, ContractWrapper, Executor};
    use lending::msg::{
        ExecuteCollateralVault, ExecuteMsg, GetBalanceResponse, InstantiateMsg, QueryMsg,
    };
    use lending::{execute, instantiate, query};
    use pyth_sdk_cw::PriceIdentifier;

    #[test]
    fn test_success_deposit_one_token_borrow_another() {
        const TOKENS_DECIMALS: u32 = 18;

        const INIT_BALANCE_ETH: u128 = 1000 * 10u128.pow(TOKENS_DECIMALS); // 1000 ETH
        const INIT_BALANCE_ATOM: u128 = 1000 * 10u128.pow(TOKENS_DECIMALS); // 1000 ATOM

        const INIT_LIQUIDATOR_BALANCE_ETH: u128 = 1_000_000 * 10u128.pow(TOKENS_DECIMALS); // 1M ETH
        const INIT_LIQUIDATOR_BALANCE_ATOM: u128 = 1_000_000 * 10u128.pow(TOKENS_DECIMALS); // 1M ATOM

        const RESERVE_AMOUNT: u128 = 1000 * 10u128.pow(TOKENS_DECIMALS);

        const DEPOSIT_AMOUNT_ETH: u128 = 200 * 10u128.pow(TOKENS_DECIMALS); // 200 ETH

        const BORROW_AMOUNT_ATOM: u128 = 300 * 10u128.pow(TOKENS_DECIMALS); // 300 ATOM

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
        const PRICE_ETH: u128 = 2000 * 10u128.pow(PRICE_DECIMALS); // 2000$/1ETH
        const PRICE_ATOM: u128 = 10 * 10u128.pow(PRICE_DECIMALS); // 10$/1ATOM

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
                        coin(10 * RESERVE_AMOUNT, "eth"),
                        coin(10 * RESERVE_AMOUNT, "atom"),
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
            &[coin(RESERVE_AMOUNT, "atom")],
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
                    margin_positions_contract: "whatever".to_string(),
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

        assert_eq!(get_price_atom.u128(), 1000000000); // 10$/1ATOM
        assert_eq!(get_price_eth.u128(), 200000000000); // 2000$/1ETH

        app.execute_contract(
            Addr::unchecked("user"),
            lending_addr.clone(),
            &ExecuteMsg::Deposit {},
            &coins(DEPOSIT_AMOUNT_ETH, "eth"),
        )
        .unwrap();

        app.execute_contract(
            Addr::unchecked("user"),
            lending_addr.clone(),
            &ExecuteMsg::ToggleCollateralSetting {
                denom: "eth".to_string(),
            },
            &[],
        )
        .unwrap();

        let user_available_to_borrow_eth: Uint128 = app
            .wrap()
            .query_wasm_smart(
                lending_addr.clone(),
                &QueryMsg::GetAvailableToBorrow {
                    address: "user".to_string(),
                    denom: "eth".to_string(),
                },
            )
            .unwrap();

        let user_available_to_borrow_atom: Uint128 = app
            .wrap()
            .query_wasm_smart(
                lending_addr.clone(),
                &QueryMsg::GetAvailableToBorrow {
                    address: "user".to_string(),
                    denom: "atom".to_string(),
                },
            )
            .unwrap();

        // max_allowed_borrow_amount_usd = DEPOSIT_AMOUNT_ETH * PRICE_ETH * LTV_ETH = 200 ETH * 2000 * 0,85 = 340000$
        // user_available_to_borrow_eth = max_allowed_borrow_amount_usd / PRICE_ETH = 340000$ / 2000 = 170 ETH
        assert_eq!(
            user_available_to_borrow_eth.u128(),
            170000000000000000000 // 170 ETH
        );

        // user_available_to_borrow_atom = max_allowed_borrow_amount_usd / PRICE_ATOM = 320000$ / 10 = 32000 ATOM
        // But not enough liquidity!! => user_available_to_borrow_atom = CONTRACT_RESERVES_ATOM == 1000 ATOM
        assert_eq!(
            user_available_to_borrow_atom.u128(),
            1000000000000000000000 // 1000 ATOM
        );

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
            &ExecuteMsg::Borrow {
                denom: "atom".to_string(),
                amount: Uint128::from(BORROW_AMOUNT_ATOM),
            },
            &[],
        )
        .unwrap();

        let user_borrowed_balance: Uint128 = app
            .wrap()
            .query_wasm_smart(
                lending_addr.clone(),
                &QueryMsg::GetUserBorrowAmountWithInterest {
                    address: "user".to_string(),
                    denom: "atom".to_string(),
                },
            )
            .unwrap();

        assert_eq!(user_borrowed_balance.u128(), BORROW_AMOUNT_ATOM);
    }
}
