#[cfg(test)]
mod tests {
    use crate::utils::success_deposit_of_diff_token_with_prices;
    use master_contract::msg::{GetReserveConfigurationResponse, QueryMsg};

    #[test]
    fn test_get_reserve_configuration() {
        const PERCENT_DECIMALS: u32 = 5;
        const LTV_ETH: u128 = 85 * 10u128.pow(PERCENT_DECIMALS); // 85%
        const LIQUIDATION_THRESHOLD_ETH: u128 = 90 * 10u128.pow(PERCENT_DECIMALS); // 90%
        const LTV_ATOM: u128 = 75 * 10u128.pow(PERCENT_DECIMALS); // 75%
        const LIQUIDATION_THRESHOLD_ATOM: u128 = 80 * 10u128.pow(PERCENT_DECIMALS); // 80%

        // contract reserves: 1000 ETH and 1000 ATOM
        // user deposited 200 ETH and 300 ATOM
        let (app, addr) = success_deposit_of_diff_token_with_prices();

        let reserve_configuration_response: GetReserveConfigurationResponse =
            app.wrap()
                .query_wasm_smart(addr.clone(), &QueryMsg::GetReserveConfiguration {})
                .unwrap();

        println!(
            "{}",
            format!(
                "{:?}",
                reserve_configuration_response.reserve_configuration
            )
        );

        assert_eq!(
            reserve_configuration_response.reserve_configuration[0]
                .denom,
            "atom".to_string()
        );
        assert_eq!(
            reserve_configuration_response.reserve_configuration[0]
                .loan_to_value_ratio,
            LTV_ATOM
        );
        assert_eq!(
            reserve_configuration_response.reserve_configuration[0]
                .liquidation_threshold,
            LIQUIDATION_THRESHOLD_ATOM
        );

        assert_eq!(
            reserve_configuration_response.reserve_configuration[1]
                .denom,
            "eth".to_string()
        );
        assert_eq!(
            reserve_configuration_response.reserve_configuration[1]
                .loan_to_value_ratio,
            LTV_ETH
        );
        assert_eq!(
            reserve_configuration_response.reserve_configuration[1]
                .liquidation_threshold,
            LIQUIDATION_THRESHOLD_ETH
        );
    }
}
