#[cfg(test)]
mod tests {
    use crate::utils::success_deposit_of_diff_token_with_prices;
    use cosmwasm_std::Addr;
    use cw_multi_test::Executor;
    use master_contract::msg::ExecuteMsg;

    #[test]
    #[should_panic(expected = "This functionality is allowed for admin only")]
    fn test_fail_set_tokens_interest_rate_model_params_if_caller_is_not_owner() {
        const PERCENT_DECIMALS: u32 = 5;

        const INTEREST_RATE_DECIMALS: u32 = 18;
        const MIN_INTEREST_RATE_ETH: u128 = 10 * 10u128.pow(INTEREST_RATE_DECIMALS); // 10%
        const SAFE_BORROW_MAX_RATE_ETH: u128 = 70 * 10u128.pow(INTEREST_RATE_DECIMALS); // 70%
        const RATE_GROWTH_FACTOR_ETH: u128 = 95 * 10u128.pow(INTEREST_RATE_DECIMALS); // 95%
        const OPTIMAL_UTILISATION_RATIO_ETH: u128 = 70 * 10u128.pow(PERCENT_DECIMALS); // 70%

        let (mut app, addr) = success_deposit_of_diff_token_with_prices();

        app.execute_contract(
            Addr::unchecked("user"),
            addr.clone(),
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
    }

    #[test]
    #[should_panic(expected = "There is no such supported token yet")]
    fn test_fail_set_tokens_interest_rate_model_params_if_token_is_not_supported() {
        const PERCENT_DECIMALS: u32 = 5;

        const INTEREST_RATE_DECIMALS: u32 = 18;
        const MIN_INTEREST_RATE_ETH: u128 = 10 * 10u128.pow(INTEREST_RATE_DECIMALS); // 10%
        const SAFE_BORROW_MAX_RATE_ETH: u128 = 70 * 10u128.pow(INTEREST_RATE_DECIMALS); // 70%
        const RATE_GROWTH_FACTOR_ETH: u128 = 95 * 10u128.pow(INTEREST_RATE_DECIMALS); // 95%
        const OPTIMAL_UTILISATION_RATIO_ETH: u128 = 70 * 10u128.pow(PERCENT_DECIMALS); // 70%

        let (mut app, addr) = success_deposit_of_diff_token_with_prices();

        app.execute_contract(
            Addr::unchecked("owner"),
            addr.clone(),
            &ExecuteMsg::SetTokenInterestRateModelParams {
                denom: "usdt".to_string(),
                min_interest_rate: MIN_INTEREST_RATE_ETH,
                safe_borrow_max_rate: SAFE_BORROW_MAX_RATE_ETH,
                rate_growth_factor: RATE_GROWTH_FACTOR_ETH,
                optimal_utilisation_ratio: OPTIMAL_UTILISATION_RATIO_ETH,
            },
            &[],
        )
        .unwrap();
    }
}
