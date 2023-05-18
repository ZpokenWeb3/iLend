#[cfg(test)]
mod tests {
    use crate::utils::success_deposit_of_diff_token_with_prices;
    use master_contract::msg::{ExecuteMsg};
    use cw_multi_test::{Executor};
    use cosmwasm_std::{Addr};

    #[test]
    #[should_panic(
        expected = "This functionality is allowed for admin only"
    )]
    fn test_fail_set_reserve_configuration_if_caller_is_not_owner() {
        const PERCENT_DECIMALS: u32 = 5;
        const LTV_ETH: u128 = 92 * 10u128.pow(PERCENT_DECIMALS); // 92%
        const LIQUIDATION_THRESHOLD_ETH: u128 = 98 * 10u128.pow(PERCENT_DECIMALS); // 98%

        let (mut app, addr) = success_deposit_of_diff_token_with_prices();

        app.execute_contract(
            Addr::unchecked("user"),
            addr.clone(),
            &ExecuteMsg::SetReserveConfiguration {
                denom: "eth".to_string(),
                loan_to_value_ratio: LTV_ETH,
                liquidation_threshold: LIQUIDATION_THRESHOLD_ETH,
            },
            &[],
        )
        .unwrap();
    }

    #[test]
    #[should_panic(
        expected = "There is no such supported token yet"
    )]
    fn test_fail_set_reserve_configuration_if_token_is_not_supported() {
        const PERCENT_DECIMALS: u32 = 5;
        const LTV_ETH: u128 = 92 * 10u128.pow(PERCENT_DECIMALS); // 92%
        const LIQUIDATION_THRESHOLD_ETH: u128 = 98 * 10u128.pow(PERCENT_DECIMALS); // 98%

        let (mut app, addr) = success_deposit_of_diff_token_with_prices();

        app.execute_contract(
            Addr::unchecked("owner"),
            addr.clone(),
            &ExecuteMsg::SetReserveConfiguration {
                denom: "usdt".to_string(),
                loan_to_value_ratio: LTV_ETH,
                liquidation_threshold: LIQUIDATION_THRESHOLD_ETH,
            },
            &[],
        )
        .unwrap();
    }
}
