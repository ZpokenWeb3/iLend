#[cfg(test)]
mod tests {
    const TOKENS_DECIMALS: u32 = 18;
    const PERCENT_DECIMALS: u32 = 5;

    const LTV_ETH: u128 = 85 * 10u128.pow(PERCENT_DECIMALS);
    const LTV_TIA: u128 = 85 * 10u128.pow(PERCENT_DECIMALS);

    const LIQUIDATION_THRESHOLD_ETH: u128 = 90 * 10u128.pow(PERCENT_DECIMALS);
    const LIQUIDATION_THRESHOLD_TIA: u128 = 90 * 10u128.pow(PERCENT_DECIMALS);

    const INTEREST_RATE_DECIMALS: u32 = 18;
    const MIN_INTEREST_RATE: u128 = 5 * 10u128.pow(INTEREST_RATE_DECIMALS);
    const SAFE_BORROW_MAX_RATE: u128 = 30 * 10u128.pow(INTEREST_RATE_DECIMALS);
    const RATE_GROWTH_FACTOR: u128 = 70 * 10u128.pow(INTEREST_RATE_DECIMALS);

    const OPTIMAL_UTILISATION_RATIO: u128 = 80 * 10u128.pow(PERCENT_DECIMALS);


    use crate::utils::success_deposit_of_one_token_setup;
    use cosmwasm_std::{Addr, Uint128};
    use cw_multi_test::Executor;
    use lending::msg::{ExecuteMsg, GetSupportedTokensResponse, QueryMsg};

    #[test]
    #[should_panic]
    fn test_user_fail_add_markets() {
        let (mut app, addr) = success_deposit_of_one_token_setup();

        app.execute_contract(
            Addr::unchecked("admin"),
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
    }

    #[test]
    #[should_panic]
    fn test_token_already_exists() {
        let (mut app, addr) = success_deposit_of_one_token_setup();

        app.execute_contract(
            Addr::unchecked("owner"),
            addr.clone(),
            &ExecuteMsg::UpdateAdmin {
                admin: "admin".to_string(),
            },
            &[],
        )
            .unwrap();

        let new_admin: String = app
            .wrap()
            .query_wasm_smart(
                addr.clone(),
                &QueryMsg::GetAdmin {},
            )
            .unwrap();

        assert_eq!(new_admin, "admin");

        app.execute_contract(
            Addr::unchecked("admin"),
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
    }

    #[test]
    fn test_add_market_success() {
        let (mut app, addr) = success_deposit_of_one_token_setup();

        app.execute_contract(
            Addr::unchecked("owner"),
            addr.clone(),
            &ExecuteMsg::UpdateAdmin {
                admin: "admin".to_string(),
            },
            &[],
        )
            .unwrap();

        let new_admin: String = app
            .wrap()
            .query_wasm_smart(
                addr.clone(),
                &QueryMsg::GetAdmin {},
            )
            .unwrap();

        assert_eq!(new_admin, "admin");

        app.execute_contract(
            Addr::unchecked("admin"),
            addr.clone(),
            &ExecuteMsg::AddMarkets {
                denom: "tia".to_string(),
                name: "Celestia".to_string(),
                symbol: "TIA".to_string(),
                decimals: TOKENS_DECIMALS as u128,
                loan_to_value_ratio: LTV_TIA,
                liquidation_threshold: LIQUIDATION_THRESHOLD_TIA,
                min_interest_rate: MIN_INTEREST_RATE,
                safe_borrow_max_rate: SAFE_BORROW_MAX_RATE,
                rate_growth_factor: RATE_GROWTH_FACTOR,
                optimal_utilisation_ratio: OPTIMAL_UTILISATION_RATIO,
            },
            &[],
        )
            .unwrap();

        let supported_tokens_response: GetSupportedTokensResponse = app
            .wrap()
            .query_wasm_smart(addr.clone(), &QueryMsg::GetSupportedTokens {})
            .unwrap();

        println!(
            "{}",
            format!("{:?}", supported_tokens_response.supported_tokens)
        );
    }
}
