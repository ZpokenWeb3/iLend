#[cfg(test)]
mod tests {
    const TOKENS_DECIMALS: u32 = 18;
    const PERCENT_DECIMALS: u32 = 5;

    const LTV_ETH: u128 = 85 * 10u128.pow(PERCENT_DECIMALS);
    const LTV_TIA: u128 = 75 * 10u128.pow(PERCENT_DECIMALS);

    const LIQUIDATION_THRESHOLD_ETH: u128 = 90 * 10u128.pow(PERCENT_DECIMALS);
    const LIQUIDATION_THRESHOLD_TIA: u128 = 90 * 10u128.pow(PERCENT_DECIMALS);

    const INTEREST_RATE_DECIMALS: u32 = 18;
    const MIN_INTEREST_RATE: u128 = 5 * 10u128.pow(INTEREST_RATE_DECIMALS);
    const SAFE_BORROW_MAX_RATE: u128 = 30 * 10u128.pow(INTEREST_RATE_DECIMALS);
    const RATE_GROWTH_FACTOR: u128 = 70 * 10u128.pow(INTEREST_RATE_DECIMALS);

    const OPTIMAL_UTILISATION_RATIO: u128 = 80 * 10u128.pow(PERCENT_DECIMALS);

    use crate::utils::success_deposit_of_one_token_setup;
    use cosmwasm_std::{coins, Addr, Uint128};
    use cw_multi_test::Executor;
    use lending::msg::QueryMsg::GetPriceFeedIds;
    use lending::msg::{
        ExecuteMsg, GetReserveConfigurationResponse, GetSupportedTokensResponse, QueryMsg,
    };
    use pyth_sdk_cw::PriceIdentifier;

    #[test]
    fn test_remove_price_ids() {
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
            .query_wasm_smart(addr.clone(), &QueryMsg::GetAdmin {})
            .unwrap();

        assert_eq!(new_admin, "admin");

        app.execute_contract(
            Addr::unchecked("admin"),
            addr.clone(),
            &ExecuteMsg::AddMarkets {
                denom: "tia".to_string(),
                name: "Celestia".to_string(),
                symbol: "TIA".to_string(),
                cw20_address: None,
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

        let price_ids: Vec<(String, PriceIdentifier)> = app
            .wrap()
            .query_wasm_smart(addr.clone(), &GetPriceFeedIds {})
            .unwrap();

        dbg!(price_ids);

        app.execute_contract(
            Addr::unchecked("admin"),
            addr.clone(),
            &ExecuteMsg::RemovePriceFeedId {
                denom: "tia".to_string(),
            },
            &[],
        )
        .unwrap();

        let price_ids_after: Vec<(String, PriceIdentifier)> = app
            .wrap()
            .query_wasm_smart(addr.clone(), &GetPriceFeedIds {})
            .unwrap();

        dbg!(price_ids_after);
    }
}
