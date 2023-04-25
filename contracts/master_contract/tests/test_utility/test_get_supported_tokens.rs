#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::success_deposit_of_diff_token_with_prices;
    use master_contract::msg::{GetSupportedTokensResponse, QueryMsg};

    #[test]
    fn test_get_supported_tokens() {
        let (mut app, addr) = success_deposit_of_diff_token_with_prices();

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
