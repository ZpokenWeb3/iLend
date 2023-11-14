#[cfg(test)]
mod tests {
    use cosmwasm_std::{coin, coins, Addr, BlockInfo, Empty, Timestamp};

    use cw_multi_test::{custom_app, ContractWrapper, Executor};
    use std::vec;

    use crate::utils::CustomMsg;
    use collateral_vault::msg::{
        InstantiateMsg as InstantiateMsgCollateralVault, QueryMsg as QueryMsgCollateralVault,
    };
    use collateral_vault::{
        execute as execute_collateral_vault, instantiate as instantiate_collateral_vault,
        query as query_collateral_vault,
    };
    use lending::msg::{ExecuteMsg, GetBalanceResponse, InstantiateMsg, QueryMsg};
    use lending::{execute, instantiate, query};
    use pyth_sdk_cw::PriceIdentifier;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn test_successful_deposits_of_one_token() {
        const TOKENS_DECIMALS: u32 = 18;

        const INIT_USER_BALANCE: u128 = 1000 * 10u128.pow(TOKENS_DECIMALS);
        const INIT_LIQUIDATOR_BALANCE_ETH: u128 = 1_000_000 * 10u128.pow(TOKENS_DECIMALS); // 1M ETH
        const INIT_LIQUIDATOR_BALANCE_ATOM: u128 = 1_000_000 * 10u128.pow(TOKENS_DECIMALS); // 1M ATOM

        const CONTRACT_RESERVES: u128 = 1000000 * 10u128.pow(TOKENS_DECIMALS);
        const FIRST_DEPOSIT_AMOUNT: u128 = 200 * 10u128.pow(TOKENS_DECIMALS);
        const SECOND_DEPOSIT_AMOUNT: u128 = 300 * 10u128.pow(TOKENS_DECIMALS);

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
                        coin(INIT_USER_BALANCE, "eth"),
                        coin(INIT_USER_BALANCE, "atom"),
                    ],
                )
                .unwrap();

            router
                .bank
                .init_balance(
                    storage,
                    &Addr::unchecked("owner"),
                    vec![
                        coin(CONTRACT_RESERVES * 100, "eth"),
                        coin(CONTRACT_RESERVES * 100, "atom"),
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
                        coin(CONTRACT_RESERVES, "eth"),
                        coin(CONTRACT_RESERVES, "atom"),
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
                &[coin(CONTRACT_RESERVES, "atom")],
                "Collateral Vault Contract",
                Some("collateral_vault".to_string()), // contract that can execute migrations
            )
            .unwrap();

        let lending_contract: String = app
            .wrap()
            .query_wasm_smart(
                collateral_contract_addr.clone(),
                &QueryMsgCollateralVault::GetLendingContract {},
            )
            .unwrap();

        assert_eq!(lending_contract, "owner".to_string());

        let margin_contract: String = app
            .wrap()
            .query_wasm_smart(
                collateral_contract_addr.clone(),
                &QueryMsgCollateralVault::GetMarginContract {},
            )
            .unwrap();

        assert_eq!(margin_contract, "whatever".to_string());

        let code = ContractWrapper::new_with_empty(execute, instantiate, query);
        let code_id = app.store_code(Box::new(code));

        let addr = app
            .instantiate_contract(
                code_id,
                Addr::unchecked("owner"),
                &InstantiateMsg {
                    is_testing: true,
                    admin: "owner".to_string(),
                    liquidator: "liquidator".to_string(),
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
                    supported_tokens: vec![(
                        "atom".to_string(),
                        "atom".to_string(),
                        "ATOM".to_string(),
                        6,
                    )],
                    reserve_configuration: vec![(
                        "atom".to_string(),
                        LTV_ATOM,
                        LIQUIDATION_THRESHOLD_ATOM,
                    )],
                    tokens_interest_rate_model_params: vec![(
                        "atom".to_string(),
                        5000000000000000000,
                        20000000000000000000,
                        100000000000000000000,
                        OPTIMAL_UTILISATION_RATIO,
                    )],
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
            Addr::unchecked("user"),
            addr.clone(),
            &ExecuteMsg::AddMarkets {
                denom: "eth".to_string(),
                name: "ethereum".to_string(),
                symbol: "ETH".to_string(),
                decimals: TOKENS_DECIMALS as u128,
                loan_to_value_ratio: LTV_ETH,
                liquidation_threshold: LIQUIDATION_THRESHOLD_ETH,
                min_interest_rate: MIN_INTEREST_RATE,
                safe_borrow_max_rate: SAFE_BORROW_MAX_RATE,
                rate_growth_factor: RATE_GROWTH_FACTOR,
                optimal_utilisation_ratio: OPTIMAL_UTILISATION_RATIO,
            },
            &[],
        )
        .unwrap();

        let collateral_vault_amount_before_deposit = app
            .wrap()
            .query_balance(collateral_contract_addr.to_string(), "eth")
            .unwrap()
            .amount
            .u128();

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

        app.execute_contract(
            Addr::unchecked("user"),
            addr.clone(),
            &ExecuteMsg::Deposit {},
            &coins(FIRST_DEPOSIT_AMOUNT, "eth"),
        )
        .unwrap();

        let collateral_vault_amount_after_first_deposit = app
            .wrap()
            .query_balance(collateral_contract_addr.to_string(), "eth")
            .unwrap()
            .amount
            .u128();

        assert_eq!(
            FIRST_DEPOSIT_AMOUNT + collateral_vault_amount_before_deposit,
            collateral_vault_amount_after_first_deposit
        );

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
            Addr::unchecked("owner"),
            addr.clone(),
            &ExecuteMsg::ToggleCollateralSetting {
                denom: "eth".to_string(),
            },
            &[],
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
            &ExecuteMsg::Deposit {},
            &coins(SECOND_DEPOSIT_AMOUNT, "eth"),
        )
        .unwrap();

        let collateral_vault_amount_after_second_deposit = app
            .wrap()
            .query_balance(collateral_contract_addr.to_string(), "eth")
            .unwrap()
            .amount
            .u128();

        assert_eq!(
            collateral_vault_amount_after_first_deposit + SECOND_DEPOSIT_AMOUNT,
            collateral_vault_amount_after_second_deposit
        );
    }
}
