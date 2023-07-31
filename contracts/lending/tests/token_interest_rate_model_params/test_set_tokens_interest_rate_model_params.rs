#[cfg(test)]
mod tests {
    use crate::utils::success_deposit_of_diff_token_with_prices;
    use cosmwasm_std::Addr;
    use cw_multi_test::Executor;
    use lending::msg::{ExecuteMsg, GetTokensInterestRateModelParamsResponse, QueryMsg};

    #[test]
    fn test_set_tokens_interest_rate_model_params() {
        const PERCENT_DECIMALS: u32 = 5;

        const INTEREST_RATE_DECIMALS: u32 = 18;
        const MIN_INTEREST_RATE_ETH: u128 = 10 * 10u128.pow(INTEREST_RATE_DECIMALS); // 10%
        const SAFE_BORROW_MAX_RATE_ETH: u128 = 70 * 10u128.pow(INTEREST_RATE_DECIMALS); // 70%
        const RATE_GROWTH_FACTOR_ETH: u128 = 95 * 10u128.pow(INTEREST_RATE_DECIMALS); // 95%
        const OPTIMAL_UTILISATION_RATIO_ETH: u128 = 70 * 10u128.pow(PERCENT_DECIMALS); // 70%

        const MIN_INTEREST_RATE_ATOM: u128 = 15 * 10u128.pow(INTEREST_RATE_DECIMALS); // 15%
        const SAFE_BORROW_MAX_RATE_ATOM: u128 = 45 * 10u128.pow(INTEREST_RATE_DECIMALS); // 45%
        const RATE_GROWTH_FACTOR_ATOM: u128 = 100 * 10u128.pow(INTEREST_RATE_DECIMALS); // 100%
        const OPTIMAL_UTILISATION_RATIO_ATOM: u128 = 45 * 10u128.pow(PERCENT_DECIMALS); // 45%

        // contract reserves: 1000 ETH and 1000 ATOM
        // user deposited 200 ETH and 300 ATOM
        let (mut app, lending_contract_addr, _collateral_contract_addr) =
            success_deposit_of_diff_token_with_prices();

        app.execute_contract(
            Addr::unchecked("owner"),
            lending_contract_addr.clone(),
            &ExecuteMsg::SetTokenInterestRateModelParams {
                denom: "eth".to_string(),
                min_interest_rate: MIN_INTEREST_RATE_ETH,
                safe_borrow_max_rate: SAFE_BORROW_MAX_RATE_ETH,
                rate_growth_factor: RATE_GROWTH_FACTOR_ETH,
                optimal_utilisation_ratio: OPTIMAL_UTILISATION_RATIO_ETH,
            },
            &[],
        )
        .unwrap();

        app.execute_contract(
            Addr::unchecked("owner"),
            lending_contract_addr.clone(),
            &ExecuteMsg::SetTokenInterestRateModelParams {
                denom: "atom".to_string(),
                min_interest_rate: MIN_INTEREST_RATE_ATOM,
                safe_borrow_max_rate: SAFE_BORROW_MAX_RATE_ATOM,
                rate_growth_factor: RATE_GROWTH_FACTOR_ATOM,
                optimal_utilisation_ratio: OPTIMAL_UTILISATION_RATIO_ATOM,
            },
            &[],
        )
        .unwrap();

        let tokens_interest_rate_model_params_response: GetTokensInterestRateModelParamsResponse =
            app.wrap()
                .query_wasm_smart(
                    lending_contract_addr.clone(),
                    &QueryMsg::GetTokensInterestRateModelParams {},
                )
                .unwrap();

        println!(
            "{}",
            format!(
                "{:?}",
                tokens_interest_rate_model_params_response.tokens_interest_rate_model_params
            )
        );

        assert_eq!(
            tokens_interest_rate_model_params_response.tokens_interest_rate_model_params[0]
                .min_interest_rate,
            MIN_INTEREST_RATE_ATOM
        ); // 5%
        assert_eq!(
            tokens_interest_rate_model_params_response.tokens_interest_rate_model_params[0]
                .safe_borrow_max_rate,
            SAFE_BORROW_MAX_RATE_ATOM
        ); // 30%
        assert_eq!(
            tokens_interest_rate_model_params_response.tokens_interest_rate_model_params[0]
                .rate_growth_factor,
            RATE_GROWTH_FACTOR_ATOM
        ); // 70%
        assert_eq!(
            tokens_interest_rate_model_params_response.tokens_interest_rate_model_params[0]
                .optimal_utilisation_ratio,
            OPTIMAL_UTILISATION_RATIO_ATOM
        ); // 80%

        assert_eq!(
            tokens_interest_rate_model_params_response.tokens_interest_rate_model_params[1]
                .min_interest_rate,
            MIN_INTEREST_RATE_ETH
        ); // 5%
        assert_eq!(
            tokens_interest_rate_model_params_response.tokens_interest_rate_model_params[1]
                .safe_borrow_max_rate,
            SAFE_BORROW_MAX_RATE_ETH
        ); // 30%
        assert_eq!(
            tokens_interest_rate_model_params_response.tokens_interest_rate_model_params[1]
                .rate_growth_factor,
            RATE_GROWTH_FACTOR_ETH
        ); // 70%
        assert_eq!(
            tokens_interest_rate_model_params_response.tokens_interest_rate_model_params[1]
                .optimal_utilisation_ratio,
            OPTIMAL_UTILISATION_RATIO_ETH
        ); // 80%
    }
}
