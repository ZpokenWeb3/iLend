#[cfg(test)]
mod tests {
    use crate::utils::success_deposit_of_diff_token_with_prices;
    use master_contract::msg::{ExecuteMsg, GetReserveConfigurationResponse, QueryMsg};
    use cw_multi_test::{Executor};
    use cosmwasm_std::{Addr};

    #[test]
    fn test_set_reserve_configuration() {
        const PERCENT_DECIMALS: u32 = 5;
        const LTV_ETH: u128 = 92 * 10u128.pow(PERCENT_DECIMALS); // 92%
        const LIQUIDATION_THRESHOLD_ETH: u128 = 98 * 10u128.pow(PERCENT_DECIMALS); // 98%
        const LTV_ATOM: u128 = 78 * 10u128.pow(PERCENT_DECIMALS); // 78%
        const LIQUIDATION_THRESHOLD_ATOM: u128 = 86 * 10u128.pow(PERCENT_DECIMALS); // 86%

        // contract reserves: 1000 ETH and 1000 ATOM
        // user deposited 200 ETH and 300 ATOM
        let (mut app, addr) = success_deposit_of_diff_token_with_prices();

        app.execute_contract(
            Addr::unchecked("owner"),
            addr.clone(),
            &ExecuteMsg::SetReserveConfiguration {
                denom: "eth".to_string(),
                loan_to_value_ratio: LTV_ETH,
                liquidation_threshold: LIQUIDATION_THRESHOLD_ETH,
            },
            &[],
        )
        .unwrap();
        
        app.execute_contract(
            Addr::unchecked("owner"),
            addr.clone(),
            &ExecuteMsg::SetReserveConfiguration {
                denom: "atom".to_string(),
                loan_to_value_ratio: LTV_ATOM,
                liquidation_threshold: LIQUIDATION_THRESHOLD_ATOM,
            },
            &[],
        )
        .unwrap();

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
