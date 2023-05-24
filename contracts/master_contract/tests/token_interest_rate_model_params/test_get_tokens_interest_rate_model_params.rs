#[cfg(test)]
mod tests {
    //     use super::*;
    use crate::utils::success_deposit_of_diff_token_with_prices;
    use master_contract::msg::{GetTokensInterestRateModelParamsResponse, QueryMsg};

    #[test]
    fn test_get_tokens_interest_rate_model_params() {
        let (app, addr) = success_deposit_of_diff_token_with_prices();

        let tokens_interest_rate_model_params_response: GetTokensInterestRateModelParamsResponse =
            app.wrap()
                .query_wasm_smart(addr.clone(), &QueryMsg::GetTokensInterestRateModelParams {})
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
            5000000000000000000
        ); // 5%
        assert_eq!(
            tokens_interest_rate_model_params_response.tokens_interest_rate_model_params[0]
                .safe_borrow_max_rate,
            30000000000000000000
        ); // 30%
        assert_eq!(
            tokens_interest_rate_model_params_response.tokens_interest_rate_model_params[0]
                .rate_growth_factor,
            70000000000000000000
        ); // 70%
        assert_eq!(
            tokens_interest_rate_model_params_response.tokens_interest_rate_model_params[0]
                .optimal_utilisation_ratio,
            8000000
        ); // 80%
    }
}
